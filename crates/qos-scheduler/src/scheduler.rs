//! QoS scheduler implementation
//!
//! Provides stream priority control and bandwidth allocation for multi-stream sessions.
//! Implements in-process allocation API for Control Plane integration.

use honeylink_core::types::StreamId;
use std::collections::HashMap;

/// QoS priority levels for stream allocation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QoSPriority {
    /// High-priority burst traffic (e.g., video streaming)
    /// Higher bandwidth, more FEC redundancy
    Burst,

    /// Normal priority (e.g., telemetry)
    /// Standard bandwidth, moderate FEC
    Normal,

    /// Low-latency traffic (e.g., control commands)
    /// Lower bandwidth, minimal FEC for speed
    Latency,
}

/// Stream mode (reliability requirement)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamMode {
    /// Reliable stream (TCP-like, retransmission)
    Reliable,

    /// Unreliable stream (UDP-like, best-effort)
    Unreliable,
}

/// Stream allocation request
#[derive(Debug, Clone)]
pub struct StreamRequest {
    pub name: String,
    pub mode: StreamMode,
    pub priority: QoSPriority,
    pub bandwidth_kbps: u32,
}

/// Stream allocation result
#[derive(Debug, Clone)]
pub struct StreamAllocation {
    pub stream_id: StreamId,
    pub name: String,
    pub connection_id: String, // Connection identifier for transport layer
    pub priority: QoSPriority,
    pub allocated_bandwidth_kbps: u32,
}

/// Allocation error types
#[derive(Debug, thiserror::Error)]
pub enum AllocationError {
    #[error("Insufficient bandwidth: requested {requested} kbps, available {available} kbps")]
    InsufficientBandwidth { requested: u32, available: u32 },

    #[error("Too many streams: requested {requested}, max {max}")]
    TooManyStreams { requested: usize, max: usize },

    #[error("Invalid stream configuration: {0}")]
    InvalidConfiguration(String),
}

pub struct QoSScheduler {
    streams: Vec<StreamId>,
    // Simple allocation state (can be enhanced later with more sophisticated algorithms)
    total_bandwidth_kbps: u32,
    allocated_bandwidth_kbps: u32,
    max_streams: usize,
}

impl QoSScheduler {
    pub fn new() -> Self {
        Self {
            streams: Vec::new(),
            total_bandwidth_kbps: 100_000, // 100 Mbps default
            allocated_bandwidth_kbps: 0,
            max_streams: 16, // Max 16 streams per session
        }
    }

    /// Creates a scheduler with custom limits
    pub fn with_limits(total_bandwidth_kbps: u32, max_streams: usize) -> Self {
        Self {
            streams: Vec::new(),
            total_bandwidth_kbps,
            allocated_bandwidth_kbps: 0,
            max_streams,
        }
    }

    pub fn add_stream(&mut self, stream_id: StreamId) {
        self.streams.push(stream_id);
    }

    /// Allocates multiple streams with QoS guarantees
    ///
    /// # Arguments
    /// * `requests` - Vector of stream allocation requests
    ///
    /// # Returns
    /// * `Ok(Vec<StreamAllocation>)` - Successful allocations
    /// * `Err(AllocationError)` - Allocation failed due to resource constraints
    ///
    /// # Allocation Strategy
    /// 1. Check total bandwidth and stream count limits
    /// 2. Allocate streams with priority: Burst > Normal > Latency
    /// 3. Assign connection IDs based on priority (conn-001, conn-002, ...)
    /// 4. Track allocated bandwidth
    ///
    /// # Example
    /// ```no_run
    /// use honeylink_qos_scheduler::{QoSScheduler, StreamRequest, QoSPriority, StreamMode};
    ///
    /// let mut scheduler = QoSScheduler::new();
    /// let requests = vec![
    ///     StreamRequest {
    ///         name: "telemetry".to_string(),
    ///         mode: StreamMode::Reliable,
    ///         priority: QoSPriority::Normal,
    ///         bandwidth_kbps: 100,
    ///     },
    ///     StreamRequest {
    ///         name: "video".to_string(),
    ///         mode: StreamMode::Unreliable,
    ///         priority: QoSPriority::Burst,
    ///         bandwidth_kbps: 5000,
    ///     },
    /// ];
    ///
    /// let allocations = scheduler.allocate_streams(&requests).unwrap();
    /// ```
    pub fn allocate_streams(
        &mut self,
        requests: &[StreamRequest],
    ) -> Result<Vec<StreamAllocation>, AllocationError> {
        // Validate stream count
        if requests.len() > self.max_streams {
            return Err(AllocationError::TooManyStreams {
                requested: requests.len(),
                max: self.max_streams,
            });
        }

        // Calculate total requested bandwidth
        let total_requested: u32 = requests.iter().map(|r| r.bandwidth_kbps).sum();
        let available = self.total_bandwidth_kbps - self.allocated_bandwidth_kbps;

        if total_requested > available {
            return Err(AllocationError::InsufficientBandwidth {
                requested: total_requested,
                available,
            });
        }

        // Sort requests by priority (Burst > Normal > Latency)
        let mut sorted_requests: Vec<(usize, &StreamRequest)> =
            requests.iter().enumerate().collect();
        sorted_requests.sort_by_key(|(_, req)| match req.priority {
            QoSPriority::Burst => 0,
            QoSPriority::Normal => 1,
            QoSPriority::Latency => 2,
        });

        // Allocate streams
        let mut allocations = Vec::with_capacity(requests.len());
        let mut conn_id_counter = 1u32;

        for (original_index, request) in sorted_requests {
            // Generate stream ID (UUIDv7 for time-ordering)
            let stream_id = StreamId::new();

            // Generate connection ID based on priority
            let connection_id = format!("conn-{:03}", conn_id_counter);
            conn_id_counter += 1;

            // Create allocation
            let allocation = StreamAllocation {
                stream_id,
                name: request.name.clone(),
                connection_id,
                priority: request.priority,
                allocated_bandwidth_kbps: request.bandwidth_kbps,
            };

            // Track stream
            self.add_stream(stream_id);
            self.allocated_bandwidth_kbps += request.bandwidth_kbps;

            allocations.push((original_index, allocation));
        }

        // Restore original order
        allocations.sort_by_key(|(idx, _)| *idx);
        let allocations: Vec<StreamAllocation> = allocations.into_iter().map(|(_, a)| a).collect();

        Ok(allocations)
    }

    /// Releases allocated resources for a stream
    pub fn release_stream(&mut self, stream_id: StreamId, bandwidth_kbps: u32) {
        self.streams.retain(|id| *id != stream_id);
        self.allocated_bandwidth_kbps = self.allocated_bandwidth_kbps.saturating_sub(bandwidth_kbps);
    }

    /// Gets current allocation statistics
    pub fn get_stats(&self) -> AllocationStats {
        AllocationStats {
            total_streams: self.streams.len(),
            max_streams: self.max_streams,
            total_bandwidth_kbps: self.total_bandwidth_kbps,
            allocated_bandwidth_kbps: self.allocated_bandwidth_kbps,
            available_bandwidth_kbps: self.total_bandwidth_kbps - self.allocated_bandwidth_kbps,
        }
    }
}

/// Allocation statistics
#[derive(Debug, Clone)]
pub struct AllocationStats {
    pub total_streams: usize,
    pub max_streams: usize,
    pub total_bandwidth_kbps: u32,
    pub allocated_bandwidth_kbps: u32,
    pub available_bandwidth_kbps: u32,
}

impl Default for QoSScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocate_streams_success() {
        let mut scheduler = QoSScheduler::new();

        let requests = vec![
            StreamRequest {
                name: "telemetry".to_string(),
                mode: StreamMode::Reliable,
                priority: QoSPriority::Normal,
                bandwidth_kbps: 100,
            },
            StreamRequest {
                name: "video".to_string(),
                mode: StreamMode::Unreliable,
                priority: QoSPriority::Burst,
                bandwidth_kbps: 5000,
            },
        ];

        let allocations = scheduler.allocate_streams(&requests).unwrap();

        assert_eq!(allocations.len(), 2);
        assert_eq!(allocations[0].name, "telemetry");
        assert_eq!(allocations[1].name, "video");

        let stats = scheduler.get_stats();
        assert_eq!(stats.total_streams, 2);
        assert_eq!(stats.allocated_bandwidth_kbps, 5100);
    }

    #[test]
    fn test_allocate_streams_insufficient_bandwidth() {
        let mut scheduler = QoSScheduler::with_limits(1000, 16);

        let requests = vec![
            StreamRequest {
                name: "video".to_string(),
                mode: StreamMode::Unreliable,
                priority: QoSPriority::Burst,
                bandwidth_kbps: 5000,
            },
        ];

        let result = scheduler.allocate_streams(&requests);
        assert!(result.is_err());

        match result.unwrap_err() {
            AllocationError::InsufficientBandwidth { requested, available } => {
                assert_eq!(requested, 5000);
                assert_eq!(available, 1000);
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_allocate_streams_too_many() {
        let mut scheduler = QoSScheduler::with_limits(100_000, 2);

        let requests = vec![
            StreamRequest {
                name: "stream1".to_string(),
                mode: StreamMode::Reliable,
                priority: QoSPriority::Normal,
                bandwidth_kbps: 100,
            },
            StreamRequest {
                name: "stream2".to_string(),
                mode: StreamMode::Reliable,
                priority: QoSPriority::Normal,
                bandwidth_kbps: 100,
            },
            StreamRequest {
                name: "stream3".to_string(),
                mode: StreamMode::Reliable,
                priority: QoSPriority::Normal,
                bandwidth_kbps: 100,
            },
        ];

        let result = scheduler.allocate_streams(&requests);
        assert!(result.is_err());

        match result.unwrap_err() {
            AllocationError::TooManyStreams { requested, max } => {
                assert_eq!(requested, 3);
                assert_eq!(max, 2);
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_release_stream() {
        let mut scheduler = QoSScheduler::new();

        let requests = vec![
            StreamRequest {
                name: "test".to_string(),
                mode: StreamMode::Reliable,
                priority: QoSPriority::Normal,
                bandwidth_kbps: 1000,
            },
        ];

        let allocations = scheduler.allocate_streams(&requests).unwrap();
        let stream_id = allocations[0].stream_id;

        scheduler.release_stream(stream_id, 1000);

        let stats = scheduler.get_stats();
        assert_eq!(stats.total_streams, 0);
        assert_eq!(stats.allocated_bandwidth_kbps, 0);
    }
}
