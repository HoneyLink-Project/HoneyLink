//! Preset QoS profiles for common use cases
//!
//! Implements the 4 preset profiles defined in spec/modules/policy-profile-engine.md:
//! - IoT Low Power (prof_iot_lowpower_v2)
//! - AR/VR Spatial (prof_arvr_spatial_v1)
//! - 8K Media (prof_media_8k_v1)
//! - Gaming Input (prof_gaming_input_v1)

use crate::profile::PolicyProfile;
use crate::types::{FecMode, PowerProfile, UseCase};
use chrono::Utc;
use semver::Version;
use std::collections::HashMap;

/// Create all preset profiles
pub fn create_presets() -> Vec<PolicyProfile> {
    vec![
        create_iot_lowpower_preset(),
        create_arvr_spatial_preset(),
        create_media_8k_preset(),
        create_gaming_input_preset(),
    ]
}

/// IoT Low Power Profile (prof_iot_lowpower_v2)
///
/// **Use Case**: IoT sensor/actuator with ultra-low power consumption
/// **Target**: 5mA average current, 200ms latency acceptable
/// **Requirements**: FR-04 QoS adjustment for battery-powered devices
pub fn create_iot_lowpower_preset() -> PolicyProfile {
    PolicyProfile {
        profile_id: "prof_iot_lowpower_v2".to_string(),
        profile_name: "IoT Low Power v2".to_string(),
        profile_version: Version::new(2, 0, 0),
        use_case: UseCase::IoT,

        // Latency: 200ms acceptable for sensor data
        latency_budget_ms: 200,

        // Bandwidth: 0.1-1.0 Mbps for sensor telemetry
        bandwidth_floor_mbps: 0.1,
        bandwidth_ceiling_mbps: 1.0,

        // FEC: NONE to minimize overhead and power consumption
        fec_mode: FecMode::None,

        // Priority: Low (1) - non-critical background traffic
        priority: 1,

        // Power: Ultra low power profile
        power_profile: PowerProfile::UltraLow,

        deprecated_after: None,
        signature: "PLACEHOLDER_SIGNATURE".to_string(), // Would be signed by Control Plane
        created_at: Utc::now(),
        updated_at: Utc::now(),
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("target_current_ma".to_string(), "5".to_string());
            meta.insert("burst_tolerance".to_string(), "3x".to_string());
            meta.insert("description".to_string(),
                "Optimized for battery-powered IoT devices with relaxed latency requirements".to_string());
            meta
        },
    }
}

/// AR/VR Spatial Sync Profile (prof_arvr_spatial_v1)
///
/// **Use Case**: Augmented/Virtual Reality spatial synchronization
/// **Target**: P99 12ms latency, 5cm spatial error, 50-200 Mbps
/// **Requirements**: FR-04 QoS for immersive experiences, UC-05 multi-user sync
pub fn create_arvr_spatial_preset() -> PolicyProfile {
    PolicyProfile {
        profile_id: "prof_arvr_spatial_v1".to_string(),
        profile_name: "AR/VR Spatial Sync v1".to_string(),
        profile_version: Version::new(1, 0, 0),
        use_case: UseCase::ArVr,

        // Latency: 12ms P99 for motion-to-photon pipeline
        latency_budget_ms: 12,

        // Bandwidth: 50-200 Mbps for 6DOF tracking + video
        bandwidth_floor_mbps: 50.0,
        bandwidth_ceiling_mbps: 200.0,

        // FEC: HEAVY to recover from wireless packet loss
        fec_mode: FecMode::Heavy,

        // Priority: Highest (7) for real-time immersive content
        priority: 7,

        // Power: High power acceptable for tethered/AC devices
        power_profile: PowerProfile::High,

        deprecated_after: None,
        signature: "PLACEHOLDER_SIGNATURE".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("spatial_error_cm".to_string(), "5".to_string());
            meta.insert("motion_to_photon_ms".to_string(), "20".to_string());
            meta.insert("jitter_tolerance_ms".to_string(), "3".to_string());
            meta.insert("description".to_string(),
                "Low-latency profile for AR/VR with spatial synchronization requirements".to_string());
            meta
        },
    }
}

/// 8K Media Streaming Profile (prof_media_8k_v1)
///
/// **Use Case**: High-resolution media streaming (8K video)
/// **Target**: 1.5 Gbps throughput, <0.1% frame drop, 50ms latency
/// **Requirements**: UC-03 high-resolution media transfer
pub fn create_media_8k_preset() -> PolicyProfile {
    PolicyProfile {
        profile_id: "prof_media_8k_v1".to_string(),
        profile_name: "8K Media Streaming v1".to_string(),
        profile_version: Version::new(1, 0, 0),
        use_case: UseCase::Media8K,

        // Latency: 50ms acceptable for buffered streaming
        latency_budget_ms: 50,

        // Bandwidth: 1000-1500 Mbps for 8K60fps HEVC
        bandwidth_floor_mbps: 1000.0,
        bandwidth_ceiling_mbps: 1500.0,

        // FEC: HEAVY to prevent frame corruption
        fec_mode: FecMode::Heavy,

        // Priority: High (6) for quality media delivery
        priority: 6,

        // Power: High power for sustained throughput
        power_profile: PowerProfile::High,

        deprecated_after: None,
        signature: "PLACEHOLDER_SIGNATURE".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("target_frame_rate".to_string(), "60".to_string());
            meta.insert("max_frame_drop_rate".to_string(), "0.001".to_string()); // 0.1%
            meta.insert("codec_hint".to_string(), "HEVC".to_string());
            meta.insert("buffer_ms".to_string(), "200".to_string());
            meta.insert("description".to_string(),
                "High-throughput profile for 8K video streaming with minimal frame loss".to_string());
            meta
        },
    }
}

/// Gaming Input Profile (prof_gaming_input_v1)
///
/// **Use Case**: Low-latency gaming input + voice chat
/// **Target**: P95 6ms input latency, 20ms audio sync tolerance
/// **Requirements**: UC-01 gaming input + audio simultaneous streams
pub fn create_gaming_input_preset() -> PolicyProfile {
    PolicyProfile {
        profile_id: "prof_gaming_input_v1".to_string(),
        profile_name: "Gaming Input + Voice v1".to_string(),
        profile_version: Version::new(1, 0, 0),
        use_case: UseCase::Gaming,

        // Latency: 6ms P95 for competitive gaming responsiveness
        latency_budget_ms: 6,

        // Bandwidth: 5-50 Mbps (input is low, voice is moderate)
        bandwidth_floor_mbps: 5.0,
        bandwidth_ceiling_mbps: 50.0,

        // FEC: LIGHT for low overhead while maintaining reliability
        fec_mode: FecMode::Light,

        // Priority: Highest (7) for real-time input
        priority: 7,

        // Power: Normal for handheld gaming devices
        power_profile: PowerProfile::Normal,

        deprecated_after: None,
        signature: "PLACEHOLDER_SIGNATURE".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("input_sampling_hz".to_string(), "1000".to_string());
            meta.insert("audio_sync_tolerance_ms".to_string(), "20".to_string());
            meta.insert("haptic_feedback".to_string(), "enabled".to_string());
            meta.insert("description".to_string(),
                "Ultra-low latency for competitive gaming input with voice chat".to_string());
            meta
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_presets_are_valid() {
        let presets = create_presets();
        assert_eq!(presets.len(), 4);

        for preset in &presets {
            assert!(
                preset.validate().is_ok(),
                "Preset {} validation failed",
                preset.profile_id
            );
        }
    }

    #[test]
    fn test_iot_preset_characteristics() {
        let profile = create_iot_lowpower_preset();

        assert_eq!(profile.profile_id, "prof_iot_lowpower_v2");
        assert_eq!(profile.use_case, UseCase::IoT);
        assert_eq!(profile.power_profile, PowerProfile::UltraLow);
        assert_eq!(profile.fec_mode, FecMode::None); // No FEC for power saving
        assert_eq!(profile.priority, 1); // Low priority
        assert!(profile.latency_budget_ms >= 100); // Relaxed latency
        assert!(profile.bandwidth_floor_mbps < 1.0); // Low bandwidth
    }

    #[test]
    fn test_arvr_preset_characteristics() {
        let profile = create_arvr_spatial_preset();

        assert_eq!(profile.profile_id, "prof_arvr_spatial_v1");
        assert_eq!(profile.use_case, UseCase::ArVr);
        assert_eq!(profile.power_profile, PowerProfile::High);
        assert_eq!(profile.fec_mode, FecMode::Heavy); // High FEC for reliability
        assert_eq!(profile.priority, 7); // Highest priority
        assert!(profile.latency_budget_ms <= 15); // Very low latency
        assert!(profile.bandwidth_floor_mbps >= 50.0); // High bandwidth
    }

    #[test]
    fn test_media8k_preset_characteristics() {
        let profile = create_media_8k_preset();

        assert_eq!(profile.profile_id, "prof_media_8k_v1");
        assert_eq!(profile.use_case, UseCase::Media8K);
        assert_eq!(profile.power_profile, PowerProfile::High);
        assert_eq!(profile.fec_mode, FecMode::Heavy);
        assert_eq!(profile.priority, 6);
        assert!(profile.bandwidth_floor_mbps >= 1000.0); // Very high bandwidth
        assert_eq!(profile.metadata.get("target_frame_rate").unwrap(), "60");
    }

    #[test]
    fn test_gaming_preset_characteristics() {
        let profile = create_gaming_input_preset();

        assert_eq!(profile.profile_id, "prof_gaming_input_v1");
        assert_eq!(profile.use_case, UseCase::Gaming);
        assert_eq!(profile.power_profile, PowerProfile::Normal);
        assert_eq!(profile.fec_mode, FecMode::Light); // Light FEC for low overhead
        assert_eq!(profile.priority, 7); // Highest priority
        assert!(profile.latency_budget_ms <= 10); // Ultra-low latency
        assert_eq!(profile.metadata.get("input_sampling_hz").unwrap(), "1000");
    }

    #[test]
    fn test_preset_id_uniqueness() {
        let presets = create_presets();
        let mut ids: Vec<String> = presets.iter().map(|p| p.profile_id.clone()).collect();
        ids.sort();
        ids.dedup();

        assert_eq!(ids.len(), presets.len(), "Preset IDs must be unique");
    }

    #[test]
    fn test_preset_use_case_coverage() {
        let presets = create_presets();
        let use_cases: Vec<UseCase> = presets.iter().map(|p| p.use_case).collect();

        assert!(use_cases.contains(&UseCase::IoT));
        assert!(use_cases.contains(&UseCase::ArVr));
        assert!(use_cases.contains(&UseCase::Media8K));
        assert!(use_cases.contains(&UseCase::Gaming));
    }
}
