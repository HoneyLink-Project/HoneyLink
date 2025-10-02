//! # Physical Adapter Layer
//!
//! Abstract physical layer drivers for Wi-Fi, 5G, THz, etc.
//!
//! This module provides:
//! - Concrete adapter implementations (WiFi6eAdapter, WiFi7Adapter, FiveGAdapter, ThzAdapter)
//! - Hot Swap capability for seamless layer switching
//! - REST/gRPC client integration (no C/C++ dependencies)
//! - Power management with 4 modes (UltraLow, Low, Normal, High)
//!
//! # C/C++ Dependency Avoidance (MOD-007 spec)
//! All physical layer communication uses process separation via network APIs (REST/gRPC),
//! eliminating direct C/C++ library dependencies.

pub mod adapter;
pub mod registry;
pub mod traits;

pub use adapter::{AdapterType, FiveGAdapter, ThzAdapter, WiFi6eAdapter, WiFi7Adapter};
pub use honeylink_transport::PhysicalLayer;
pub use registry::{AdapterRegistry, HotSwapStrategy};
pub use traits::{LayerConfig, LinkStatus};
