//! # Experience Layer
//!
//! SDK API and WASM bindings for UI integration.

pub mod sdk;

pub use sdk::HoneyLinkSdk;

// WASM bindings
#[cfg(target_arch = "wasm32")]
pub mod wasm;
