//! # Policy & Profile Engine
//!
//! QoS policy and profile management with:
//! - QoSPolicyUpdate schema with SemVer versioning
//! - Profile CRUD with Ed25519 signature verification
//! - Preset profiles for IoT/AR-VR/8K/Gaming use cases
//! - Event bus for policy distribution with fallback
//!
//! **Module Specification**: MOD-002-POLICY-ENGINE
//! **Requirements**: FR-04 (QoS adjustment), FR-06 (Profile templates)
//!
//! ## C/C++ Dependency Status
//! ✅ Pure Rust implementation - No C/C++ dependencies
//! - Uses `ed25519-dalek` (pure Rust)
//! - Uses `semver` (pure Rust)
//! - Uses `tokio` (pure Rust)
//! - Uses `serde` (pure Rust)

pub mod error;
pub mod event_bus;
pub mod policy;
pub mod presets;
pub mod profile;
pub mod telemetry;
pub mod types;

// Re-export commonly used types
pub use error::{PolicyError, Result};
pub use event_bus::{PolicyEvent, PolicyEventBus};
pub use policy::PolicyEngine;
pub use presets::create_presets;
pub use profile::{InMemoryProfileStorage, PolicyProfile, ProfileStorage};
pub use telemetry::PolicyTelemetry;
pub use types::{FecMode, PowerProfile, Priority, QoSPolicyUpdate, UseCase};
