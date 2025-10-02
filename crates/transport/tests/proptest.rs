//! Property-based tests for transport layer components
//!
//! Tests using proptest to verify invariants across a wide range of inputs.

use honeylink_transport::{FecEncoder, FecStrategy, Packet, WeightedFairQueuing};
use proptest::prelude::*;

proptest! {
    /// Property: FEC None mode should preserve data exactly
    #[test]
    fn prop_fec_none_preserves_data(data in prop::collection::vec(any::<u8>(), 1..1000)) {
        let encoder = FecEncoder::new(FecStrategy::None, 1024);
        let shards = encoder.encode(&data).unwrap();

        // Decode should return exact original data
        let shard_opts: Vec<Option<Vec<u8>>> = shards.into_iter().map(Some).collect();
        let decoded = encoder.decode(&shard_opts, data.len()).unwrap();

        prop_assert_eq!(decoded, data);
    }

    /// Property: FEC Light mode should recover from single shard loss
    #[test]
    fn prop_fec_light_recovers_single_loss(
        data in prop::collection::vec(any::<u8>(), 100..1000),
        lost_shard_idx in 0usize..5
    ) {
        let encoder = FecEncoder::new(FecStrategy::Light, 128);
        let shards = encoder.encode(&data).unwrap();

        if lost_shard_idx >= shards.len() {
            return Ok(());
        }

        // Simulate losing one shard
        let mut shard_opts: Vec<Option<Vec<u8>>> = shards.into_iter().map(Some).collect();
        shard_opts[lost_shard_idx] = None;

        // Should still recover original data
        let decoded = encoder.decode(&shard_opts, data.len()).unwrap();
        prop_assert_eq!(decoded, data);
    }

    /// Property: Packet priority must be 0-7
    #[test]
    fn prop_packet_priority_valid(
        data in prop::collection::vec(any::<u8>(), 1..100),
        priority in 0u8..=7
    ) {
        let packet = Packet::new(data.clone(), priority);
        prop_assert!(packet.is_ok());

        let p = packet.unwrap();
        prop_assert_eq!(p.priority, priority);
        prop_assert_eq!(p.data, data);
    }

    /// Property: Packet with invalid priority should fail
    #[test]
    fn prop_packet_invalid_priority_fails(
        data in prop::collection::vec(any::<u8>(), 1..100),
        priority in 8u8..=255
    ) {
        let packet = Packet::new(data, priority);
        prop_assert!(packet.is_err());
    }

    /// Property: WFQ queue depth should never exceed max
    #[test]
    fn prop_wfq_respects_max_depth(
        packets in prop::collection::vec((any::<Vec<u8>>(), 0u8..=7), 0..100)
    ) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let wfq = WeightedFairQueuing::new();

            for (data, priority) in packets {
                let packet = match Packet::new(data, priority) {
                    Ok(p) => p,
                    Err(_) => continue,
                };

                let _ = wfq.enqueue(packet).await;

                // Total depth should never exceed MAX_QUEUE_DEPTH (10000)
                let depth = wfq.total_queue_depth().await;
                prop_assert!(depth <= 10000);
            }

            Ok(())
        })?;
    }

    /// Property: WFQ dequeue preserves all enqueued packets (when not full)
    #[test]
    fn prop_wfq_preserves_packets(
        packets in prop::collection::vec((any::<Vec<u8>>(), 0u8..=7), 1..50)
    ) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let wfq = WeightedFairQueuing::new();
            let mut enqueued_count = 0;

            for (data, priority) in packets {
                let packet = match Packet::new(data, priority) {
                    Ok(p) => p,
                    Err(_) => continue,
                };

                if wfq.enqueue(packet).await.is_ok() {
                    enqueued_count += 1;
                }
            }

            // Dequeue all packets
            let mut dequeued_count = 0;
            while wfq.dequeue().await.is_some() {
                dequeued_count += 1;
            }

            // All enqueued packets should be dequeued
            prop_assert_eq!(enqueued_count, dequeued_count);

            Ok(())
        })?;
    }

    /// Property: FEC Strategy selection is consistent with loss rate
    #[test]
    fn prop_fec_strategy_selection(loss_rate in 0.0f32..1.0f32) {
        let strategy = FecStrategy::select_for_loss_rate(loss_rate);

        if loss_rate < 0.05 {
            prop_assert_eq!(strategy, FecStrategy::None);
        } else if loss_rate < 0.10 {
            prop_assert_eq!(strategy, FecStrategy::Light);
        } else {
            prop_assert_eq!(strategy, FecStrategy::Heavy);
        }
    }

    /// Property: FEC overhead percentage is monotonically increasing
    #[test]
    fn prop_fec_overhead_monotonic(_unit in any::<()>()) {
        let none_overhead = FecStrategy::None.overhead_percent();
        let light_overhead = FecStrategy::Light.overhead_percent();
        let heavy_overhead = FecStrategy::Heavy.overhead_percent();

        prop_assert!(none_overhead <= light_overhead);
        prop_assert!(light_overhead <= heavy_overhead);
    }

    /// Property: Packet size is always equal to data length
    #[test]
    fn prop_packet_size_equals_data_len(
        data in prop::collection::vec(any::<u8>(), 0..1000),
        priority in 0u8..=7
    ) {
        let packet = Packet::new(data.clone(), priority).unwrap();
        prop_assert_eq!(packet.size(), data.len());
    }
}
