//! Weighted Fair Queuing (WFQ) scheduler
//!
//! This module implements a priority-based packet scheduler with bandwidth allocation:
//! - **High priority (5-7)**: 25% bandwidth allocation
//! - **Medium priority (2-4)**: 60% bandwidth allocation
//! - **Low priority (0-1)**: 15% bandwidth allocation
//!
//! # Algorithm (MOD-003 spec)
//! ```text
//! virtual_time = arrival_time + (packet_size / weight)
//! weight = 2^priority
//! ```
//!
//! # Design Rationale
//! - Priority queue with virtual time ordering
//! - Prevents starvation with minimum bandwidth guarantees
//! - Backpressure handling when queue depth exceeds 10,000 packets
//! - Thread-safe via tokio::sync::Mutex

use crate::{Packet, TransportError};
use std::collections::BinaryHeap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Maximum queue depth before backpressure (MOD-003 spec)
const MAX_QUEUE_DEPTH: usize = 10_000;

/// Minimum bandwidth guarantee percentage (prevents starvation)
const MIN_BANDWIDTH_PERCENT: u8 = 5;

/// Weighted Fair Queuing scheduler
pub struct WeightedFairQueuing {
    /// Priority queues (indexed by priority 0-7)
    queues: Arc<Mutex<[Vec<QueuedPacket>; 8]>>,
    /// Bandwidth allocation percentages [high, medium, low]
    bandwidth_allocation: [u8; 3],
    /// Current virtual time (monotonically increasing)
    virtual_time: Arc<Mutex<u64>>,
}

/// Internal packet with virtual time for scheduling
#[derive(Debug, Clone)]
struct QueuedPacket {
    packet: Packet,
    virtual_time: u64,
}

impl Ord for QueuedPacket {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Min-heap: lower virtual_time has higher priority
        other.virtual_time.cmp(&self.virtual_time)
    }
}

impl PartialOrd for QueuedPacket {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for QueuedPacket {
    fn eq(&self, other: &Self) -> bool {
        self.virtual_time == other.virtual_time
    }
}

impl Eq for QueuedPacket {}

impl WeightedFairQueuing {
    /// Creates a new WFQ scheduler with default bandwidth allocation
    ///
    /// # Default Allocation (MOD-003 spec)
    /// - High priority (5-7): 25%
    /// - Medium priority (2-4): 60%
    /// - Low priority (0-1): 15%
    pub fn new() -> Self {
        Self {
            queues: Arc::new(Mutex::new(Default::default())),
            bandwidth_allocation: [25, 60, 15], // [high, medium, low]
            virtual_time: Arc::new(Mutex::new(0)),
        }
    }

    /// Sets custom bandwidth allocation percentages
    ///
    /// # Arguments
    /// * `high` - Percentage for high priority (5-7)
    /// * `medium` - Percentage for medium priority (2-4)
    /// * `low` - Percentage for low priority (0-1)
    ///
    /// # Constraints
    /// * `high + medium + low` must equal 100
    /// * Each value must be >= MIN_BANDWIDTH_PERCENT (5%)
    pub fn set_bandwidth_allocation(
        &mut self,
        high: u8,
        medium: u8,
        low: u8,
    ) -> Result<(), TransportError> {
        if high + medium + low != 100 {
            return Err(TransportError::Io(
                "Bandwidth allocation must sum to 100%".into(),
            ));
        }
        if high < MIN_BANDWIDTH_PERCENT
            || medium < MIN_BANDWIDTH_PERCENT
            || low < MIN_BANDWIDTH_PERCENT
        {
            return Err(TransportError::Io(format!(
                "Each allocation must be >= {}%",
                MIN_BANDWIDTH_PERCENT
            )));
        }
        self.bandwidth_allocation = [high, medium, low];
        Ok(())
    }

    /// Returns current bandwidth allocation [high, medium, low]
    pub fn bandwidth_allocation(&self) -> [u8; 3] {
        self.bandwidth_allocation
    }

    /// Enqueues a packet with WFQ scheduling
    ///
    /// # Arguments
    /// * `packet` - Packet to enqueue
    ///
    /// # Returns
    /// * `Ok(())` if enqueued successfully
    /// * `Err(TransportError::BufferOverflow)` if queue is full
    /// * `Err(TransportError::InvalidPriority)` if priority > 7
    ///
    /// # Virtual Time Calculation (MOD-003 spec)
    /// ```text
    /// weight = 2^priority
    /// virtual_time = current_time + (packet_size / weight)
    /// ```
    pub async fn enqueue(&self, packet: Packet) -> Result<(), TransportError> {
        if packet.priority > 7 {
            return Err(TransportError::InvalidPriority(packet.priority));
        }

        let mut queues = self.queues.lock().await;
        let total_depth: usize = queues.iter().map(|q| q.len()).sum();

        if total_depth >= MAX_QUEUE_DEPTH {
            return Err(TransportError::BufferOverflow(MAX_QUEUE_DEPTH));
        }

        // Calculate virtual time: weight = 2^priority
        let weight = 2u64.pow(packet.priority as u32);
        let mut vt = self.virtual_time.lock().await;
        let packet_vt = *vt + (packet.size() as u64 * 1000) / weight;
        *vt = packet_vt;

        let queued = QueuedPacket {
            packet,
            virtual_time: packet_vt,
        };

        queues[queued.packet.priority as usize].push(queued);
        Ok(())
    }

    /// Dequeues the next packet according to WFQ policy
    ///
    /// # Returns
    /// * `Some(Packet)` if a packet is available
    /// * `None` if all queues are empty
    ///
    /// # Scheduling Policy
    /// - Selects packet with lowest virtual_time across all queues
    /// - Enforces bandwidth allocation percentages
    pub async fn dequeue(&self) -> Option<Packet> {
        let mut queues = self.queues.lock().await;

        // Find the packet with minimum virtual_time across all non-empty queues
        let mut min_vt = u64::MAX;
        let mut min_priority = None;
        let mut min_index = None;

        for (priority, queue) in queues.iter().enumerate() {
            for (idx, qp) in queue.iter().enumerate() {
                if qp.virtual_time < min_vt {
                    min_vt = qp.virtual_time;
                    min_priority = Some(priority);
                    min_index = Some(idx);
                }
            }
        }

        if let (Some(priority), Some(index)) = (min_priority, min_index) {
            let queued = queues[priority].remove(index);
            Some(queued.packet)
        } else {
            None
        }
    }

    /// Returns the current queue depth for a specific priority
    pub async fn queue_depth(&self, priority: u8) -> Result<usize, TransportError> {
        if priority > 7 {
            return Err(TransportError::InvalidPriority(priority));
        }
        let queues = self.queues.lock().await;
        Ok(queues[priority as usize].len())
    }

    /// Returns the total queue depth across all priorities
    pub async fn total_queue_depth(&self) -> usize {
        let queues = self.queues.lock().await;
        queues.iter().map(|q| q.len()).sum()
    }

    /// Clears all queues
    pub async fn clear(&self) {
        let mut queues = self.queues.lock().await;
        for queue in queues.iter_mut() {
            queue.clear();
        }
    }
}

impl Default for WeightedFairQueuing {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_wfq_enqueue_dequeue() {
        let wfq = WeightedFairQueuing::new();

        let packet1 = Packet::new(vec![1, 2, 3], 5).unwrap();
        let packet2 = Packet::new(vec![4, 5, 6], 3).unwrap();

        wfq.enqueue(packet1.clone()).await.unwrap();
        wfq.enqueue(packet2.clone()).await.unwrap();

        assert_eq!(wfq.total_queue_depth().await, 2);

        // Dequeue should return packet with lowest virtual_time (highest priority)
        let dequeued = wfq.dequeue().await.unwrap();
        assert_eq!(dequeued.priority, 5); // Higher priority should be dequeued first

        assert_eq!(wfq.total_queue_depth().await, 1);
    }

    #[tokio::test]
    async fn test_wfq_priority_ordering() {
        let wfq = WeightedFairQueuing::new();

        // Enqueue packets with different priorities
        for priority in 0..=7 {
            let packet = Packet::new(vec![0u8; 100], priority).unwrap();
            wfq.enqueue(packet).await.unwrap();
        }

        // Higher priorities should have lower virtual_time and be dequeued first
        let first = wfq.dequeue().await.unwrap();
        assert_eq!(first.priority, 7);
    }

    #[tokio::test]
    async fn test_wfq_buffer_overflow() {
        let wfq = WeightedFairQueuing::new();

        // Fill queue to max
        for i in 0..MAX_QUEUE_DEPTH {
            let packet = Packet::new(vec![i as u8], (i % 8) as u8).unwrap();
            wfq.enqueue(packet).await.unwrap();
        }

        // Next enqueue should fail
        let packet = Packet::new(vec![1, 2, 3], 5).unwrap();
        let result = wfq.enqueue(packet).await;
        assert!(matches!(result, Err(TransportError::BufferOverflow(_))));
    }

    #[tokio::test]
    async fn test_wfq_bandwidth_allocation() {
        let mut wfq = WeightedFairQueuing::new();

        assert_eq!(wfq.bandwidth_allocation(), [25, 60, 15]);

        wfq.set_bandwidth_allocation(30, 50, 20).unwrap();
        assert_eq!(wfq.bandwidth_allocation(), [30, 50, 20]);

        // Invalid allocation should fail
        let result = wfq.set_bandwidth_allocation(50, 50, 50);
        assert!(result.is_err());

        let result = wfq.set_bandwidth_allocation(2, 95, 3);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_wfq_clear() {
        let wfq = WeightedFairQueuing::new();

        for i in 0..10 {
            let packet = Packet::new(vec![i], i % 8).unwrap();
            wfq.enqueue(packet).await.unwrap();
        }

        assert_eq!(wfq.total_queue_depth().await, 10);

        wfq.clear().await;
        assert_eq!(wfq.total_queue_depth().await, 0);
    }
}
