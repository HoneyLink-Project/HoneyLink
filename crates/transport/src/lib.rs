//! # Transport Abstraction Layer
//!
//! Transport abstraction with FEC and WFQ support.

pub mod fec;
pub mod wfq;

pub use fec::FecStrategy;
pub use wfq::WeightedFairQueuing;
