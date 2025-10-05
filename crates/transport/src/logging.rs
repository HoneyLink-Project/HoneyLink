//! Logging utilities for HoneyLink Transport
//!
//! Provides tracing subscriber initialization for applications using HoneyLink.
//! Libraries should remain subscriber-agnostic; only binaries/examples configure subscribers.

use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize tracing subscriber with sensible defaults
///
/// Configures structured logging with:
/// - Environment-based filtering (RUST_LOG)
/// - Timestamp formatting
/// - Target module names
/// - Compact output format
///
/// # Environment Variables
/// - `RUST_LOG`: Filter directives (default: "info")
///   - Example: `RUST_LOG=honeylink_transport=debug,honeylink_discovery=trace`
///
/// # Example
/// ```no_run
/// use honeylink_transport::logging::init_tracing;
///
/// #[tokio::main]
/// async fn main() {
///     init_tracing();
///     // Your application code here
/// }
/// ```
///
/// # Panics
/// Panics if subscriber is already initialized (call once per process)
pub fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_target(true)
                .with_level(true)
                .with_thread_ids(false)
                .compact(),
        )
        .init();
}

/// Initialize tracing subscriber with custom filter
///
/// Same as `init_tracing()` but allows programmatic filter control.
///
/// # Parameters
/// - `filter`: Log level filter (e.g., "debug", "info,honeylink_transport=trace")
///
/// # Example
/// ```no_run
/// use honeylink_transport::logging::init_tracing_with_filter;
///
/// #[tokio::main]
/// async fn main() {
///     init_tracing_with_filter("debug");
///     // Your application code here
/// }
/// ```
///
/// # Panics
/// Panics if subscriber is already initialized or filter is invalid
pub fn init_tracing_with_filter(filter: &str) {
    let env_filter = EnvFilter::new(filter);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_target(true)
                .with_level(true)
                .with_thread_ids(false)
                .compact(),
        )
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_tracing_idempotence() {
        // Tracing can only be initialized once per test process
        // This test verifies the API compiles but doesn't actually initialize
        // to avoid conflicts with other tests
        let _ = EnvFilter::new("info");
    }

    #[test]
    fn test_custom_filter_parsing() {
        // Test that filter creation doesn't panic with valid syntax
        let filter = EnvFilter::new("debug,honeylink_transport=trace");
        // EnvFilter doesn't expose its internal state, so we just verify construction succeeds
        let _ = filter;
    }
}
