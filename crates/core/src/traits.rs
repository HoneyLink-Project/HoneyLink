//! Common traits for HoneyLink modules

use crate::Result;

/// Trait for components that require lifecycle management
pub trait Lifecycle {
    /// Initialize the component
    fn initialize(&mut self) -> Result<()>;

    /// Shutdown the component gracefully
    fn shutdown(&mut self) -> Result<()>;

    /// Check if the component is healthy
    fn health_check(&self) -> Result<()>;
}

/// Trait for components that can be validated
pub trait Validate {
    /// Validate the component's state
    fn validate(&self) -> Result<()>;
}
