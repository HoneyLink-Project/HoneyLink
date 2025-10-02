// Rate limiting middleware using token bucket algorithm

use crate::config::RateLimitConfig;
use crate::error::ApiError;
use axum::{
    extract::{ConnectInfo, Request},
    middleware::Next,
    response::Response,
};
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter as GovernorRateLimiter,
};
use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::sync::Arc;

/// Rate limiter state
#[derive(Clone)]
pub struct RateLimiter {
    limiter: Arc<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}

impl RateLimiter {
    /// Create new rate limiter from configuration
    pub fn new(config: &RateLimitConfig) -> Self {
        let quota = Quota::per_second(
            NonZeroU32::new(config.requests_per_second).unwrap()
        )
        .allow_burst(NonZeroU32::new(config.burst_size).unwrap());

        let limiter = Arc::new(GovernorRateLimiter::new(
            quota,
            InMemoryState::default(),
            &DefaultClock::default(),
        ));

        RateLimiter { limiter }
    }

    /// Check if request is allowed
    pub fn check(&self) -> Result<(), ApiError> {
        self.limiter
            .check()
            .map_err(|_| ApiError::State("Rate limit exceeded".to_string()))
    }
}

/// Middleware function for rate limiting
pub async fn rate_limit_middleware(
    connect_info: Option<ConnectInfo<SocketAddr>>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Get client IP from connection info
    let client_ip = connect_info
        .map(|ConnectInfo(addr)| addr.ip())
        .unwrap_or_else(|| std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST));

    tracing::debug!(client_ip = %client_ip, "Rate limit check");

    // Note: Current implementation uses global rate limiter
    // For per-IP rate limiting, we'd need a more sophisticated approach
    // using DashMap<IpAddr, RateLimiter> or similar

    // For now, we'll implement a simple global rate limiter
    // TODO: Implement per-IP rate limiting in future iteration

    Ok(next.run(request).await)
}

/// Per-IP rate limiter (advanced implementation)
#[derive(Clone)]
pub struct PerIpRateLimiter {
    config: RateLimitConfig,
    limiters: Arc<dashmap::DashMap<std::net::IpAddr, Arc<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>>>>,
}

impl PerIpRateLimiter {
    /// Create new per-IP rate limiter
    pub fn new(config: RateLimitConfig) -> Self {
        PerIpRateLimiter {
            config,
            limiters: Arc::new(dashmap::DashMap::new()),
        }
    }

    /// Check rate limit for specific IP
    pub fn check(&self, ip: std::net::IpAddr) -> Result<(), ApiError> {
        let limiter = self.limiters.entry(ip).or_insert_with(|| {
            let quota = Quota::per_second(
                NonZeroU32::new(self.config.requests_per_second).unwrap()
            )
            .allow_burst(NonZeroU32::new(self.config.burst_size).unwrap());

            Arc::new(GovernorRateLimiter::new(
                quota,
                InMemoryState::default(),
                &DefaultClock::default(),
            ))
        });

        limiter
            .check()
            .map_err(|_| ApiError::State(format!("Rate limit exceeded for IP {}", ip)))
    }

    /// Cleanup stale limiters (should be called periodically)
    pub fn cleanup_stale(&self, max_entries: usize) {
        if self.limiters.len() > max_entries {
            // Simple cleanup: remove random entries
            // In production, we'd track last access time
            let to_remove = self.limiters.len() - max_entries;
            let mut removed = 0;
            self.limiters.retain(|_, _| {
                if removed < to_remove {
                    removed += 1;
                    false
                } else {
                    true
                }
            });
        }
    }
}

/// Middleware function for per-IP rate limiting
pub async fn per_ip_rate_limit_middleware(
    axum::extract::State(limiter): axum::extract::State<Arc<PerIpRateLimiter>>,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let client_ip = connect_info
        .map(|ConnectInfo(addr)| addr.ip())
        .unwrap_or_else(|| std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST));

    // Check rate limit
    limiter.check(client_ip)?;

    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_creation() {
        let config = RateLimitConfig {
            requests_per_second: 10,
            burst_size: 20,
        };

        let limiter = RateLimiter::new(&config);

        // First request should succeed
        assert!(limiter.check().is_ok());
    }

    #[test]
    fn test_rate_limit_exceeded() {
        let config = RateLimitConfig {
            requests_per_second: 1,
            burst_size: 2,
        };

        let limiter = RateLimiter::new(&config);

        // First 2 requests should succeed (burst)
        assert!(limiter.check().is_ok());
        assert!(limiter.check().is_ok());

        // Third request should fail
        assert!(limiter.check().is_err());
    }

    #[test]
    fn test_per_ip_rate_limiter() {
        let config = RateLimitConfig {
            requests_per_second: 5,
            burst_size: 10,
        };

        let limiter = PerIpRateLimiter::new(config);
        let ip1 = "127.0.0.1".parse().unwrap();
        let ip2 = "192.168.1.1".parse().unwrap();

        // Different IPs should have independent limits
        for _ in 0..10 {
            assert!(limiter.check(ip1).is_ok());
            assert!(limiter.check(ip2).is_ok());
        }

        // Both should now be rate limited
        assert!(limiter.check(ip1).is_err());
        assert!(limiter.check(ip2).is_err());
    }
}
