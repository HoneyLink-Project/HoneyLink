//! # Physical Adapter Layer
//!
//! Abstract physical layer drivers for Wi-Fi, 5G, THz, etc.

pub mod adapter;
pub mod traits;

pub use adapter::PhysicalAdapter;
pub use traits::PhysicalLayer;
