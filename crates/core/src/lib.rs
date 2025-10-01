//! # HoneyLink Core
//!
//! Common types, traits, and error definitions shared across all HoneyLink modules.
//!
//! ## Module Structure
//!
//! - `types`: Core type definitions (DeviceId, SessionId, etc.)
//! - `traits`: Common traits for all modules
//! - `error`: Unified error types
//! - `events`: Event definitions for the event bus

pub mod error;
pub mod events;
pub mod traits;
pub mod types;

pub use error::{Error, Result};
