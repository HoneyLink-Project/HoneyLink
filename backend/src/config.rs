// Configuration management for Control Plane API

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server bind address
    pub host: String,
    /// Server port
    pub port: u16,
    /// Enable TLS
    pub tls_enabled: bool,
    /// TLS certificate path (PEM)
    pub tls_cert_path: Option<PathBuf>,
    /// TLS private key path (PEM)
    pub tls_key_path: Option<PathBuf>,
    /// Client certificate CA path (for mTLS)
    pub client_ca_path: Option<PathBuf>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 7843,
            tls_enabled: false,
            tls_cert_path: None,
            tls_key_path: None,
            client_ca_path: None,
        }
    }
}

/// JWT authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// JWT signing algorithm (RS256, ES256, EdDSA)
    pub algorithm: String,
    /// Public key path for JWT verification (PEM)
    pub public_key_path: PathBuf,
    /// Expected issuer
    pub issuer: String,
    /// Expected audience
    pub audience: String,
    /// Token TTL in seconds (default 300 = 5 minutes)
    pub ttl_seconds: u64,
}

impl Default for JwtConfig {
    fn default() -> Self {
        JwtConfig {
            algorithm: "ES256".to_string(),
            public_key_path: PathBuf::from("./keys/jwt_public.pem"),
            issuer: "honeylink-idp".to_string(),
            audience: "honeylink-api".to_string(),
            ttl_seconds: 300,
        }
    }
}

/// CORS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    /// Allowed origins (e.g., ["https://console.honeylink.local"])
    pub allowed_origins: Vec<String>,
    /// Allow credentials
    pub allow_credentials: bool,
    /// Max age for preflight cache (seconds)
    pub max_age_seconds: u64,
}

impl Default for CorsConfig {
    fn default() -> Self {
        CorsConfig {
            allowed_origins: vec!["https://localhost:3000".to_string()],
            allow_credentials: true,
            max_age_seconds: 3600,
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Requests per second per IP
    pub requests_per_second: u32,
    /// Burst size
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        RateLimitConfig {
            requests_per_second: 100,
            burst_size: 200,
        }
    }
}

/// OpenTelemetry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelConfig {
    /// OTLP endpoint
    pub endpoint: String,
    /// Service name
    pub service_name: String,
    /// Service version
    pub service_version: String,
    /// Environment (dev, staging, prod)
    pub environment: String,
}

impl Default for OtelConfig {
    fn default() -> Self {
        OtelConfig {
            endpoint: "http://localhost:4317".to_string(),
            service_name: "honeylink-control-plane".to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            environment: "dev".to_string(),
        }
    }
}

/// Complete application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub jwt: JwtConfig,
    pub cors: CorsConfig,
    pub rate_limit: RateLimitConfig,
    pub otel: OtelConfig,
    /// Session endpoint for device pairing response
    pub session_endpoint: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            server: ServerConfig::default(),
            jwt: JwtConfig::default(),
            cors: CorsConfig::default(),
            rate_limit: RateLimitConfig::default(),
            otel: OtelConfig::default(),
            session_endpoint: Some("quic://127.0.0.1:7843".to_string()),
        }
    }
}

impl AppConfig {
    /// Load configuration from file
    pub fn from_file(path: &str) -> Result<Self, config::ConfigError> {
        let settings = config::Config::builder()
            .add_source(config::File::with_name(path))
            .add_source(config::Environment::with_prefix("HONEYLINK").separator("__"))
            .build()?;

        settings.try_deserialize()
    }

    /// Load configuration with defaults (for testing)
    pub fn default_with_env() -> Result<Self, config::ConfigError> {
        let settings = config::Config::builder()
            .add_source(config::Environment::with_prefix("HONEYLINK").separator("__"))
            .build()?;

        let mut app_config = AppConfig::default();

        // Override with environment variables if present
        if let Ok(host) = settings.get_string("server.host") {
            app_config.server.host = host;
        }
        if let Ok(port) = settings.get_int("server.port") {
            app_config.server.port = port as u16;
        }

        Ok(app_config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 7843);
        assert_eq!(config.jwt.ttl_seconds, 300);
        assert_eq!(config.rate_limit.requests_per_second, 100);
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("[server]"));
        assert!(toml_str.contains("[jwt]"));
        assert!(toml_str.contains("[cors]"));
    }
}
