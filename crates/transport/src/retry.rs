//! Retry logic and error handling for transport operations
//!
//! This module implements exponential backoff retry strategies as per MOD-003 spec:
//! - Max Retry: 3 attempts
//! - Backoff: Exponential (100ms, 200ms, 400ms)
//! - Dead Letter Queue: After 3 failures
//!
//! # Design Rationale
//! - Graceful degradation to handle transient failures
//! - Circuit breaker pattern for persistent failures
//! - Telemetry integration for retry monitoring

use crate::TransportError;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// Retry policy for send operations
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial backoff duration
    pub initial_backoff: Duration,
    /// Backoff multiplier for exponential backoff
    pub backoff_multiplier: f32,
    /// Maximum backoff duration
    pub max_backoff: Duration,
}

impl RetryPolicy {
    /// Creates the default retry policy (MOD-003 spec)
    ///
    /// - Max retries: 3
    /// - Initial backoff: 100ms
    /// - Backoff multiplier: 2.0 (exponential)
    /// - Max backoff: 1000ms
    pub fn default_transport() -> Self {
        Self {
            max_retries: 3,
            initial_backoff: Duration::from_millis(100),
            backoff_multiplier: 2.0,
            max_backoff: Duration::from_millis(1000),
        }
    }

    /// Calculates backoff duration for a given attempt
    ///
    /// # Formula
    /// ```text
    /// backoff = min(initial_backoff * multiplier^attempt, max_backoff)
    /// ```
    pub fn backoff_duration(&self, attempt: u32) -> Duration {
        let multiplier = self.backoff_multiplier.powi(attempt as i32);
        let duration_ms = (self.initial_backoff.as_millis() as f32 * multiplier) as u64;
        Duration::from_millis(duration_ms.min(self.max_backoff.as_millis() as u64))
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::default_transport()
    }
}

/// Retry executor for async operations
pub struct RetryExecutor {
    policy: RetryPolicy,
    retry_count: Arc<AtomicU64>,
    success_count: Arc<AtomicU64>,
    failure_count: Arc<AtomicU64>,
}

impl RetryExecutor {
    /// Creates a new retry executor with the given policy
    pub fn new(policy: RetryPolicy) -> Self {
        Self {
            policy,
            retry_count: Arc::new(AtomicU64::new(0)),
            success_count: Arc::new(AtomicU64::new(0)),
            failure_count: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Executes an async operation with retry logic
    ///
    /// # Arguments
    /// * `operation` - Async closure that returns Result<T, TransportError>
    ///
    /// # Returns
    /// * `Ok(T)` if operation succeeded (within max_retries)
    /// * `Err(TransportError)` if all retries exhausted
    ///
    /// # Example
    /// ```ignore
    /// let executor = RetryExecutor::new(RetryPolicy::default_transport());
    /// let result = executor.execute(|| async {
    ///     adapter.send_packet(&packet).await
    /// }).await;
    /// ```
    pub async fn execute<F, Fut, T>(&self, mut operation: F) -> Result<T, TransportError>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, TransportError>>,
    {
        let mut attempt: u32 = 0;

        loop {
            match operation().await {
                Ok(result) => {
                    self.success_count.fetch_add(1, Ordering::Relaxed);
                    if attempt > 0 {
                        self.retry_count.fetch_add(attempt as u64, Ordering::Relaxed);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    if attempt >= self.policy.max_retries {
                        self.failure_count.fetch_add(1, Ordering::Relaxed);
                        return Err(e);
                    }

                    // Check if error is retryable
                    if !is_retryable_error(&e) {
                        self.failure_count.fetch_add(1, Ordering::Relaxed);
                        return Err(e);
                    }

                    // Exponential backoff
                    let backoff = self.policy.backoff_duration(attempt);
                    sleep(backoff).await;

                    attempt += 1;
                }
            }
        }
    }

    /// Returns the total number of retry attempts
    pub fn retry_count(&self) -> u64 {
        self.retry_count.load(Ordering::Relaxed)
    }

    /// Returns the total number of successful operations
    pub fn success_count(&self) -> u64 {
        self.success_count.load(Ordering::Relaxed)
    }

    /// Returns the total number of failed operations (after retries exhausted)
    pub fn failure_count(&self) -> u64 {
        self.failure_count.load(Ordering::Relaxed)
    }

    /// Returns the success rate (0.0 to 1.0)
    pub fn success_rate(&self) -> f32 {
        let total = self.success_count() + self.failure_count();
        if total == 0 {
            return 1.0;
        }
        self.success_count() as f32 / total as f32
    }
}

impl Default for RetryExecutor {
    fn default() -> Self {
        Self::new(RetryPolicy::default_transport())
    }
}

/// Checks if a TransportError is retryable
///
/// # Retryable Errors
/// - Timeout
/// - BufferOverflow (temporary congestion)
/// - Io (network errors)
///
/// # Non-Retryable Errors
/// - LinkDown (requires Hot Swap)
/// - FecDecodingFailed (data corruption)
/// - InvalidPriority (client error)
fn is_retryable_error(error: &TransportError) -> bool {
    match error {
        TransportError::Timeout(_) => true,
        TransportError::BufferOverflow(_) => true,
        TransportError::Io(_) => true,
        TransportError::AdapterError(_) => true,
        TransportError::LinkDown => false,
        TransportError::FecDecodingFailed(_) => false,
        TransportError::InvalidPriority(_) => false,
    }
}

/// Circuit breaker state for graceful degradation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, operations proceed normally
    Closed,
    /// Circuit is open, operations fail fast
    Open,
    /// Circuit is half-open, testing if service recovered
    HalfOpen,
}

/// Circuit breaker for graceful degradation
pub struct CircuitBreaker {
    state: Arc<tokio::sync::RwLock<CircuitState>>,
    failure_threshold: u32,
    success_threshold: u32,
    timeout: Duration,
    failure_count: Arc<AtomicU64>,
    success_count: Arc<AtomicU64>,
}

impl CircuitBreaker {
    /// Creates a new circuit breaker
    ///
    /// # Arguments
    /// * `failure_threshold` - Number of consecutive failures to open circuit
    /// * `success_threshold` - Number of consecutive successes to close circuit
    /// * `timeout` - Duration to wait before transitioning to HalfOpen
    pub fn new(failure_threshold: u32, success_threshold: u32, timeout: Duration) -> Self {
        Self {
            state: Arc::new(tokio::sync::RwLock::new(CircuitState::Closed)),
            failure_threshold,
            success_threshold,
            timeout,
            failure_count: Arc::new(AtomicU64::new(0)),
            success_count: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Returns the current circuit state
    pub async fn state(&self) -> CircuitState {
        *self.state.read().await
    }

    /// Executes an operation through the circuit breaker
    pub async fn execute<F, Fut, T>(&self, operation: F) -> Result<T, TransportError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, TransportError>>,
    {
        let state = self.state().await;

        match state {
            CircuitState::Open => {
                return Err(TransportError::AdapterError(
                    "Circuit breaker is open".into(),
                ));
            }
            CircuitState::HalfOpen => {
                // Allow one probe request through
            }
            CircuitState::Closed => {
                // Normal operation
            }
        }

        match operation().await {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(e) => {
                self.on_failure().await;
                Err(e)
            }
        }
    }

    async fn on_success(&self) {
        self.success_count.fetch_add(1, Ordering::Relaxed);
        self.failure_count.store(0, Ordering::Relaxed);

        let state = self.state().await;
        if state == CircuitState::HalfOpen {
            let success = self.success_count.load(Ordering::Relaxed);
            if success >= self.success_threshold as u64 {
                *self.state.write().await = CircuitState::Closed;
                self.success_count.store(0, Ordering::Relaxed);
            }
        }
    }

    async fn on_failure(&self) {
        self.failure_count.fetch_add(1, Ordering::Relaxed);
        self.success_count.store(0, Ordering::Relaxed);

        let failures = self.failure_count.load(Ordering::Relaxed);
        if failures >= self.failure_threshold as u64 {
            *self.state.write().await = CircuitState::Open;

            // Schedule transition to HalfOpen
            let state = self.state.clone();
            let timeout = self.timeout;
            tokio::spawn(async move {
                sleep(timeout).await;
                *state.write().await = CircuitState::HalfOpen;
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_policy_backoff() {
        let policy = RetryPolicy::default_transport();

        assert_eq!(policy.backoff_duration(0), Duration::from_millis(100));
        assert_eq!(policy.backoff_duration(1), Duration::from_millis(200));
        assert_eq!(policy.backoff_duration(2), Duration::from_millis(400));
    }

    #[test]
    fn test_is_retryable_error() {
        assert!(is_retryable_error(&TransportError::Timeout(
            Duration::from_secs(5)
        )));
        assert!(is_retryable_error(&TransportError::BufferOverflow(10000)));
        assert!(!is_retryable_error(&TransportError::LinkDown));
        assert!(!is_retryable_error(&TransportError::InvalidPriority(8)));
    }

    #[tokio::test]
    async fn test_retry_executor_success() {
        let executor = RetryExecutor::new(RetryPolicy::default_transport());

        let mut attempt = 0;
        let result = executor
            .execute(|| async {
                attempt += 1;
                if attempt < 2 {
                    Err(TransportError::Timeout(Duration::from_secs(5)))
                } else {
                    Ok(42)
                }
            })
            .await;

        assert_eq!(result, Ok(42));
        assert_eq!(executor.success_count(), 1);
        assert!(executor.retry_count() > 0);
    }

    #[tokio::test]
    async fn test_retry_executor_failure() {
        let executor = RetryExecutor::new(RetryPolicy::default_transport());

        let result = executor
            .execute(|| async { Err(TransportError::Timeout(Duration::from_secs(5))) })
            .await;

        assert!(result.is_err());
        assert_eq!(executor.failure_count(), 1);
    }

    #[tokio::test]
    async fn test_circuit_breaker_open() {
        let breaker = CircuitBreaker::new(2, 1, Duration::from_millis(100));

        // First failure
        let _ = breaker
            .execute(|| async { Err::<(), _>(TransportError::LinkDown) })
            .await;

        assert_eq!(breaker.state().await, CircuitState::Closed);

        // Second failure - opens circuit
        let _ = breaker
            .execute(|| async { Err::<(), _>(TransportError::LinkDown) })
            .await;

        assert_eq!(breaker.state().await, CircuitState::Open);

        // Circuit is open - fail fast
        let result = breaker.execute(|| async { Ok(42) }).await;
        assert!(matches!(result, Err(TransportError::AdapterError(_))));
    }
}
