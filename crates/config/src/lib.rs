//! HoneyLink Configuration System
//!
//! This module provides a TOML-based configuration system for HoneyLink applications.
//! Configuration can be loaded from files, environment variables, or built with defaults.
//!
//! # Configuration Sources (Priority Order)
//!
//! 1. **Environment Variables**: `HONEYLINK_*` prefixed variables override all
//! 2. **Config File**: `honeylink.toml` in current directory or `~/.config/honeylink/`
//! 3. **Defaults**: Sensible defaults for development
//!
//! # Example honeylink.toml
//!
//! ```toml
//! [transport]
//! listen_address = "0.0.0.0:5000"
//! max_connections = 1000
//! connection_timeout_secs = 30
//! enable_quic = true
//! enable_webrtc = false
//!
//! [qos]
//! max_bandwidth_mbps = 100
//! priority_levels = 8
//! default_priority = 4
//!
//! [discovery]
//! enable_mdns = true
//! enable_manual = true
//! discovery_timeout_secs = 10
//!
//! [logging]
//! level = "info"
//! format = "compact"
//! enable_file_logging = false
//!
//! [telemetry]
//! enabled = false
//! otlp_endpoint = "http://localhost:4317"
//! service_name = "honeylink"
//! ```
//!
//! # Environment Variable Overrides
//!
//! ```bash
//! export HONEYLINK_TRANSPORT_LISTEN_ADDRESS="127.0.0.1:6000"
//! export HONEYLINK_LOGGING_LEVEL="debug"
//! export HONEYLINK_TELEMETRY_ENABLED="true"
//! ```

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;

/// Errors that can occur during configuration loading
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Configuration file not found
    #[error("Configuration file not found: {0}")]
    FileNotFound(PathBuf),

    /// Failed to read configuration file
    #[error("Failed to read config file {path}: {source}")]
    ReadError {
        path: PathBuf,
        source: std::io::Error,
    },

    /// Failed to parse TOML
    #[error("Failed to parse TOML in {path}: {source}")]
    ParseError {
        path: PathBuf,
        source: toml::de::Error,
    },

    /// Invalid configuration value
    #[error("Invalid configuration: {0}")]
    ValidationError(String),

    /// Environment variable parsing error
    #[error("Failed to parse environment variables: {0}")]
    EnvError(#[from] envy::Error),
}

/// Main configuration structure for HoneyLink
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Transport layer configuration
    pub transport: TransportConfig,
    /// QoS scheduler configuration
    pub qos: QosConfig,
    /// Discovery mechanism configuration
    pub discovery: DiscoveryConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Telemetry configuration (optional)
    pub telemetry: TelemetryConfig,
}

/// Transport layer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TransportConfig {
    /// Address to listen on (e.g., "0.0.0.0:5000")
    pub listen_address: String,
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    /// Connection timeout in seconds
    pub connection_timeout_secs: u64,
    /// Enable QUIC transport
    pub enable_quic: bool,
    /// Enable WebRTC transport (not yet implemented)
    pub enable_webrtc: bool,
    /// QUIC idle timeout in seconds
    pub quic_idle_timeout_secs: u64,
    /// Maximum number of streams per connection
    pub max_streams_per_connection: u64,
}

/// QoS scheduler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct QosConfig {
    /// Maximum bandwidth in Mbps
    pub max_bandwidth_mbps: u64,
    /// Number of priority levels (1-8)
    pub priority_levels: u8,
    /// Default priority for streams without explicit priority (1-8)
    pub default_priority: u8,
    /// Enable bandwidth enforcement
    pub enable_bandwidth_enforcement: bool,
}

/// Discovery mechanism configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DiscoveryConfig {
    /// Enable mDNS discovery
    pub enable_mdns: bool,
    /// Enable manual peer addition
    pub enable_manual: bool,
    /// Discovery timeout in seconds
    pub discovery_timeout_secs: u64,
    /// Service name for mDNS advertisement
    pub mdns_service_name: String,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    /// Log level (error, warn, info, debug, trace)
    pub level: String,
    /// Log format (compact, pretty, json)
    pub format: String,
    /// Enable file logging
    pub enable_file_logging: bool,
    /// Log file path (if file logging enabled)
    pub log_file_path: Option<String>,
}

/// Telemetry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TelemetryConfig {
    /// Enable telemetry collection
    pub enabled: bool,
    /// OTLP endpoint for telemetry export
    pub otlp_endpoint: String,
    /// Service name for telemetry
    pub service_name: String,
    /// Service version
    pub service_version: String,
    /// Environment (development, staging, production)
    pub environment: String,
    /// Metrics export interval in seconds
    pub metrics_export_interval_secs: u64,
    /// Trace sampling ratio (0.0 to 1.0)
    pub trace_sampling_ratio: f64,
}

// Default implementations
impl Default for Config {
    fn default() -> Self {
        Self {
            transport: TransportConfig::default(),
            qos: QosConfig::default(),
            discovery: DiscoveryConfig::default(),
            logging: LoggingConfig::default(),
            telemetry: TelemetryConfig::default(),
        }
    }
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            listen_address: "0.0.0.0:5000".to_string(),
            max_connections: 1000,
            connection_timeout_secs: 30,
            enable_quic: true,
            enable_webrtc: false,
            quic_idle_timeout_secs: 300,
            max_streams_per_connection: 100,
        }
    }
}

impl Default for QosConfig {
    fn default() -> Self {
        Self {
            max_bandwidth_mbps: 100,
            priority_levels: 8,
            default_priority: 4,
            enable_bandwidth_enforcement: true,
        }
    }
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            enable_mdns: true,
            enable_manual: true,
            discovery_timeout_secs: 10,
            mdns_service_name: "_honeylink._tcp".to_string(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "compact".to_string(),
            enable_file_logging: false,
            log_file_path: None,
        }
    }
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            otlp_endpoint: "http://localhost:4317".to_string(),
            service_name: "honeylink".to_string(),
            service_version: "0.1.0".to_string(),
            environment: "development".to_string(),
            metrics_export_interval_secs: 10,
            trace_sampling_ratio: 1.0,
        }
    }
}

impl Config {
    /// Load configuration with the following priority:
    /// 1. Environment variables (HONEYLINK_*)
    /// 2. Config file (if exists)
    /// 3. Defaults
    pub fn load() -> Result<Self, ConfigError> {
        // Start with defaults
        let mut config = Config::default();

        // Try to load from default config file locations
        if let Some(config_path) = Self::find_config_file() {
            config = Self::load_from_file(&config_path)?;
        }

        // Override with environment variables
        config = Self::apply_env_overrides(config)?;

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    /// Load configuration from a specific file
    pub fn load_from_file(path: &Path) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Err(ConfigError::FileNotFound(path.to_path_buf()));
        }

        let content = fs::read_to_string(path).map_err(|source| ConfigError::ReadError {
            path: path.to_path_buf(),
            source,
        })?;

        let config: Config =
            toml::from_str(&content).map_err(|source| ConfigError::ParseError {
                path: path.to_path_buf(),
                source,
            })?;

        Ok(config)
    }

    /// Find config file in standard locations
    /// 1. ./honeylink.toml (current directory)
    /// 2. ~/.config/honeylink/honeylink.toml
    fn find_config_file() -> Option<PathBuf> {
        // Check current directory
        let local_config = PathBuf::from("honeylink.toml");
        if local_config.exists() {
            return Some(local_config);
        }

        // Check user config directory
        if let Some(home_dir) = dirs::home_dir() {
            let user_config = home_dir.join(".config").join("honeylink").join("honeylink.toml");
            if user_config.exists() {
                return Some(user_config);
            }
        }

        None
    }

    /// Apply environment variable overrides
    /// Environment variables are prefixed with HONEYLINK_ and use __ for nesting
    /// Example: HONEYLINK_TRANSPORT__LISTEN_ADDRESS="127.0.0.1:6000"
    fn apply_env_overrides(mut config: Config) -> Result<Self, ConfigError> {
        // Note: envy doesn't support nested struct overrides well
        // For now, we'll implement manual environment variable parsing
        // This is a simplified implementation - a production system would use a more robust approach

        if let Ok(level) = std::env::var("HONEYLINK_LOGGING_LEVEL") {
            config.logging.level = level;
        }

        if let Ok(enabled) = std::env::var("HONEYLINK_TELEMETRY_ENABLED") {
            config.telemetry.enabled = enabled.parse().unwrap_or(false);
        }

        if let Ok(address) = std::env::var("HONEYLINK_TRANSPORT_LISTEN_ADDRESS") {
            config.transport.listen_address = address;
        }

        Ok(config)
    }

    /// Validate configuration values
    fn validate(&self) -> Result<(), ConfigError> {
        // Validate QoS priority levels
        if self.qos.priority_levels == 0 || self.qos.priority_levels > 8 {
            return Err(ConfigError::ValidationError(
                "qos.priority_levels must be between 1 and 8".to_string(),
            ));
        }

        if self.qos.default_priority == 0 || self.qos.default_priority > self.qos.priority_levels {
            return Err(ConfigError::ValidationError(format!(
                "qos.default_priority must be between 1 and {}",
                self.qos.priority_levels
            )));
        }

        // Validate logging level
        let valid_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_levels.contains(&self.logging.level.as_str()) {
            return Err(ConfigError::ValidationError(format!(
                "logging.level must be one of: {}",
                valid_levels.join(", ")
            )));
        }

        // Validate telemetry sampling ratio
        if self.telemetry.trace_sampling_ratio < 0.0 || self.telemetry.trace_sampling_ratio > 1.0
        {
            return Err(ConfigError::ValidationError(
                "telemetry.trace_sampling_ratio must be between 0.0 and 1.0".to_string(),
            ));
        }

        Ok(())
    }

    /// Save configuration to a file
    pub fn save_to_file(&self, path: &Path) -> Result<(), ConfigError> {
        let toml_string = toml::to_string_pretty(self).map_err(|e| {
            ConfigError::ValidationError(format!("Failed to serialize config: {}", e))
        })?;

        fs::write(path, toml_string).map_err(|source| ConfigError::ReadError {
            path: path.to_path_buf(),
            source,
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.transport.listen_address, "0.0.0.0:5000");
        assert_eq!(config.qos.priority_levels, 8);
        assert_eq!(config.logging.level, "info");
        assert!(!config.telemetry.enabled);
    }

    #[test]
    fn test_load_from_toml() {
        let toml_content = r#"
[transport]
listen_address = "127.0.0.1:6000"
max_connections = 500

[logging]
level = "debug"
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();

        let config = Config::load_from_file(temp_file.path()).unwrap();
        assert_eq!(config.transport.listen_address, "127.0.0.1:6000");
        assert_eq!(config.transport.max_connections, 500);
        assert_eq!(config.logging.level, "debug");
    }

    #[test]
    fn test_validation_invalid_priority_levels() {
        let mut config = Config::default();
        config.qos.priority_levels = 0;
        assert!(config.validate().is_err());

        config.qos.priority_levels = 9;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_invalid_log_level() {
        let mut config = Config::default();
        config.logging.level = "invalid".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_invalid_sampling_ratio() {
        let mut config = Config::default();
        config.telemetry.trace_sampling_ratio = 1.5;
        assert!(config.validate().is_err());

        config.telemetry.trace_sampling_ratio = -0.1;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let config = Config::default();
        let temp_file = NamedTempFile::new().unwrap();

        config.save_to_file(temp_file.path()).unwrap();
        let loaded_config = Config::load_from_file(temp_file.path()).unwrap();

        assert_eq!(config.transport.listen_address, loaded_config.transport.listen_address);
        assert_eq!(config.qos.max_bandwidth_mbps, loaded_config.qos.max_bandwidth_mbps);
    }
}
