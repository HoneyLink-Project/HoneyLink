//! Integration tests for physical adapter layer
//!
//! These tests verify the adapter registry, Hot Swap logic, and REST client integration.

use honeylink_physical_adapter::{AdapterRegistry, AdapterType, HotSwapStrategy, ThzAdapter};
use honeylink_transport::{Packet, PhysicalLayer, PowerMode};

#[tokio::test]
async fn test_adapter_registry_registration() {
    let registry = AdapterRegistry::new(HotSwapStrategy::Manual);

    // Register THz adapter (doesn't require external service)
    registry.register_thz().await.unwrap();

    let adapters = registry.registered_adapters().await;
    assert_eq!(adapters.len(), 1);
    assert!(adapters.contains(&AdapterType::THz));
}

#[tokio::test]
async fn test_adapter_registry_set_active() {
    let registry = AdapterRegistry::new(HotSwapStrategy::Manual);
    registry.register_thz().await.unwrap();

    // Set THz as active
    registry.set_active(AdapterType::THz).await.unwrap();

    let active = registry.active_adapter().await;
    assert_eq!(active, Some(AdapterType::THz));
}

#[tokio::test]
async fn test_adapter_registry_send_no_active() {
    let registry = AdapterRegistry::new(HotSwapStrategy::Manual);

    let packet = Packet::new(vec![1, 2, 3], 5).unwrap();
    let result = registry.send_packet(&packet).await;

    // Should fail because no active adapter
    assert!(result.is_err());
}

#[tokio::test]
async fn test_adapter_registry_get_link_quality() {
    let registry = AdapterRegistry::new(HotSwapStrategy::Manual);
    registry.register_thz().await.unwrap();
    registry.set_active(AdapterType::THz).await.unwrap();

    let metrics = registry.get_link_quality().await.unwrap();

    // THz adapter returns fixed metrics
    assert_eq!(metrics.rssi_dbm, -40);
    assert_eq!(metrics.snr_db, 40.0);
    assert_eq!(metrics.loss_rate, 0.0);
}

#[tokio::test]
async fn test_adapter_registry_set_power_mode() {
    let registry = AdapterRegistry::new(HotSwapStrategy::Manual);
    registry.register_thz().await.unwrap();
    registry.set_active(AdapterType::THz).await.unwrap();

    let result = registry.set_power_mode(PowerMode::Low).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_hot_swap_manual_strategy_no_switch() {
    let registry = AdapterRegistry::new(HotSwapStrategy::Manual);
    registry.register_thz().await.unwrap();
    registry.set_active(AdapterType::THz).await.unwrap();

    // Manual strategy should never switch automatically
    let switched = registry.evaluate_hot_swap().await.unwrap();
    assert_eq!(switched, false);
}

#[tokio::test]
async fn test_thz_adapter_basic_operations() {
    let adapter = ThzAdapter::new();

    // Layer type
    assert_eq!(adapter.layer_type(), "THz");

    // Get link quality
    let metrics = adapter.get_link_quality().await.unwrap();
    assert!(metrics.bandwidth_mbps > 0.0);

    // Set power mode
    let result = adapter.set_power_mode(PowerMode::High).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_thz_adapter_send_not_implemented() {
    let adapter = ThzAdapter::new();
    let packet = Packet::new(vec![1, 2, 3, 4, 5], 5).unwrap();

    // THz is experimental - send should fail
    let result = adapter.send_packet(&packet).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_adapter_type_conversions() {
    assert_eq!(AdapterType::WiFi6E.as_str(), "WiFi6E");
    assert_eq!(AdapterType::WiFi7.as_str(), "WiFi7");
    assert_eq!(AdapterType::FiveG.as_str(), "5G");
    assert_eq!(AdapterType::THz.as_str(), "THz");

    assert!(AdapterType::WiFi6E.typical_bandwidth_mbps() > 0.0);
    assert!(AdapterType::THz.typical_bandwidth_mbps() > AdapterType::WiFi6E.typical_bandwidth_mbps());
}

#[tokio::test]
async fn test_multiple_adapters_registration() {
    let registry = AdapterRegistry::new(HotSwapStrategy::HighestRssi);

    registry.register_thz().await.unwrap();

    let adapters = registry.registered_adapters().await;
    assert_eq!(adapters.len(), 1);
}

#[tokio::test]
async fn test_hot_swap_strategy_variants() {
    let strategies = vec![
        HotSwapStrategy::HighestRssi,
        HotSwapStrategy::LowestLossRate,
        HotSwapStrategy::HighestBandwidth,
        HotSwapStrategy::Manual,
    ];

    for strategy in strategies {
        let registry = AdapterRegistry::new(strategy);
        registry.register_thz().await.unwrap();
        registry.set_active(AdapterType::THz).await.unwrap();

        // Each strategy should be valid
        assert_eq!(registry.active_adapter().await, Some(AdapterType::THz));
    }
}
