//! GATT (Generic Attribute Profile) protocol definitions for BLE discovery
//!
//! Defines HoneyLink-specific GATT Service and Characteristics for device discovery
//! and pairing preparation over Bluetooth Low Energy.
//!
//! # Protocol Design
//!
//! - **Service UUID**: `0000FE00-0000-1000-8000-00805F9B34FB`
//! - **Characteristics**:
//!   - Device Info (0xFE01): Read-only, contains device_id, device_name, device_type
//!   - Pairing State (0xFE02): Read/Write, contains pairing status and session info
//!
//! # Security
//!
//! All characteristics should be accessed only over encrypted BLE connections
//! (LE Secure Connections with LESC pairing).

use crate::types::DeviceType;
use serde::{Deserialize, Serialize};

/// HoneyLink GATT Service UUID
///
/// Base: 0000FE00-0000-1000-8000-00805F9B34FB
pub const HONEYLINK_SERVICE_UUID: &str = "0000FE00-0000-1000-8000-00805F9B34FB";

/// Device Info Characteristic UUID (Read-only)
///
/// Contains: device_id, device_name, device_type
pub const DEVICE_INFO_CHAR_UUID: &str = "0000FE01-0000-1000-8000-00805F9B34FB";

/// Pairing State Characteristic UUID (Read/Write)
///
/// Contains: pairing status, session nonce, protocol version
pub const PAIRING_STATE_CHAR_UUID: &str = "0000FE02-0000-1000-8000-00805F9B34FB";

/// Maximum GATT characteristic value size (BLE constraint)
///
/// Standard BLE MTU is 23 bytes, minus 3 bytes ATT overhead = 20 bytes payload
pub const MAX_GATT_VALUE_SIZE: usize = 20;

/// Device information exposed via GATT Device Info Characteristic
///
/// Serialized format (binary, little-endian):
/// - device_id: 8 bytes (truncated SHA256 of full ID)
/// - device_type: 1 byte (enum)
/// - reserved: 11 bytes (for future use, zero-filled)
///
/// Total: 20 bytes (fits in single BLE packet)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GattDeviceInfo {
    /// Truncated device ID (first 8 bytes of SHA256)
    ///
    /// Full device_id is exchanged after pairing via secure channel
    pub device_id_short: [u8; 8],

    /// Device type enum
    pub device_type: DeviceType,

    /// Reserved for future protocol extensions
    #[serde(skip)]
    reserved: [u8; 11],
}

impl GattDeviceInfo {
    /// Create new GATT device info from full device ID and type
    ///
    /// # Arguments
    ///
    /// * `device_id` - Full device identifier (will be hashed and truncated)
    /// * `device_type` - Device category
    ///
    /// # Examples
    ///
    /// ```
    /// use honeylink_discovery::gatt::GattDeviceInfo;
    /// use honeylink_discovery::DeviceType;
    ///
    /// let info = GattDeviceInfo::new("DEV-LAPTOP-001", DeviceType::Desktop);
    /// assert_eq!(info.device_type, DeviceType::Desktop);
    /// ```
    pub fn new(device_id: &str, device_type: DeviceType) -> Self {
        use sha2::{Digest, Sha256};

        // Hash full device_id and take first 8 bytes for BLE advertisement
        let mut hasher = Sha256::new();
        hasher.update(device_id.as_bytes());
        let hash = hasher.finalize();
        let mut device_id_short = [0u8; 8];
        device_id_short.copy_from_slice(&hash[..8]);

        Self {
            device_id_short,
            device_type,
            reserved: [0u8; 11],
        }
    }

    /// Serialize to binary format for GATT characteristic value
    ///
    /// Format: [device_id_short(8) | device_type(1) | reserved(11)]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(MAX_GATT_VALUE_SIZE);
        bytes.extend_from_slice(&self.device_id_short);
        bytes.push(self.device_type.to_u8());
        bytes.extend_from_slice(&self.reserved);
        bytes
    }

    /// Deserialize from GATT characteristic value
    ///
    /// # Errors
    ///
    /// Returns error if data is invalid or too short
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        if data.len() < 9 {
            return Err(format!(
                "Invalid GATT device info: expected at least 9 bytes, got {}",
                data.len()
            ));
        }

        let mut device_id_short = [0u8; 8];
        device_id_short.copy_from_slice(&data[..8]);

        let device_type = DeviceType::from_u8(data[8]);

        let mut reserved = [0u8; 11];
        if data.len() >= MAX_GATT_VALUE_SIZE {
            reserved.copy_from_slice(&data[9..MAX_GATT_VALUE_SIZE]);
        }

        Ok(Self {
            device_id_short,
            device_type,
            reserved,
        })
    }
}

/// Pairing state for GATT Pairing State Characteristic
///
/// Serialized format (binary):
/// - state: 1 byte (0=idle, 1=discovering, 2=paired)
/// - session_nonce: 16 bytes (random, for replay protection)
/// - protocol_version: 1 byte
/// - reserved: 2 bytes
///
/// Total: 20 bytes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GattPairingState {
    /// Current pairing state
    pub state: PairingState,

    /// Session nonce for replay protection
    ///
    /// Generated randomly when pairing starts, verified in subsequent messages
    pub session_nonce: [u8; 16],

    /// Protocol version (currently 1)
    pub protocol_version: u8,

    /// Reserved for future use
    #[serde(skip)]
    reserved: [u8; 2],
}

/// Pairing state enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum PairingState {
    /// No active pairing
    Idle = 0,

    /// Discovery in progress
    Discovering = 1,

    /// Successfully paired
    Paired = 2,
}

impl PairingState {
    /// Convert to u8 representation
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Convert from u8 (with fallback to Idle)
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::Discovering,
            2 => Self::Paired,
            _ => Self::Idle,
        }
    }
}

impl GattPairingState {
    /// Create new pairing state
    ///
    /// # Arguments
    ///
    /// * `state` - Initial pairing state
    /// * `session_nonce` - Random nonce for replay protection
    pub fn new(state: PairingState, session_nonce: [u8; 16]) -> Self {
        Self {
            state,
            session_nonce,
            protocol_version: 1,
            reserved: [0u8; 2],
        }
    }

    /// Create new pairing state with random nonce
    pub fn new_with_random_nonce(state: PairingState) -> Self {
        use rand::RngCore;
        let mut nonce = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut nonce);
        Self::new(state, nonce)
    }

    /// Serialize to binary format
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(MAX_GATT_VALUE_SIZE);
        bytes.push(self.state.to_u8());
        bytes.extend_from_slice(&self.session_nonce);
        bytes.push(self.protocol_version);
        bytes.extend_from_slice(&self.reserved);
        bytes
    }

    /// Deserialize from binary
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        if data.len() < 18 {
            return Err(format!(
                "Invalid GATT pairing state: expected at least 18 bytes, got {}",
                data.len()
            ));
        }

        let state = PairingState::from_u8(data[0]);

        let mut session_nonce = [0u8; 16];
        session_nonce.copy_from_slice(&data[1..17]);

        let protocol_version = data[17];

        let mut reserved = [0u8; 2];
        if data.len() >= MAX_GATT_VALUE_SIZE {
            reserved.copy_from_slice(&data[18..MAX_GATT_VALUE_SIZE]);
        }

        Ok(Self {
            state,
            session_nonce,
            protocol_version,
            reserved,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gatt_device_info_serialization() {
        let info = GattDeviceInfo::new("DEV-TEST-001", DeviceType::Desktop);
        let bytes = info.to_bytes();

        assert_eq!(bytes.len(), MAX_GATT_VALUE_SIZE);

        let decoded = GattDeviceInfo::from_bytes(&bytes).unwrap();
        assert_eq!(decoded.device_id_short, info.device_id_short);
        assert_eq!(decoded.device_type, DeviceType::Desktop);
    }

    #[test]
    fn test_gatt_device_info_device_type_variants() {
        let types = [
            DeviceType::Desktop,
            DeviceType::Mobile,
            DeviceType::Iot,
            DeviceType::Server,
        ];

        for device_type in types {
            let info = GattDeviceInfo::new("TEST", device_type);
            let bytes = info.to_bytes();
            let decoded = GattDeviceInfo::from_bytes(&bytes).unwrap();
            assert_eq!(decoded.device_type, device_type);
        }
    }

    #[test]
    fn test_gatt_device_info_invalid_data() {
        let short_data = vec![0u8; 5];
        let result = GattDeviceInfo::from_bytes(&short_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_pairing_state_serialization() {
        let nonce = [42u8; 16];
        let state = GattPairingState::new(PairingState::Discovering, nonce);
        let bytes = state.to_bytes();

        assert_eq!(bytes.len(), MAX_GATT_VALUE_SIZE);
        assert_eq!(bytes[0], 1); // Discovering = 1

        let decoded = GattPairingState::from_bytes(&bytes).unwrap();
        assert_eq!(decoded.state, PairingState::Discovering);
        assert_eq!(decoded.session_nonce, nonce);
        assert_eq!(decoded.protocol_version, 1);
    }

    #[test]
    fn test_pairing_state_random_nonce() {
        let state1 = GattPairingState::new_with_random_nonce(PairingState::Idle);
        let state2 = GattPairingState::new_with_random_nonce(PairingState::Idle);

        // Random nonces should be different
        assert_ne!(state1.session_nonce, state2.session_nonce);
    }

    #[test]
    fn test_pairing_state_enum_conversion() {
        assert_eq!(PairingState::Idle.to_u8(), 0);
        assert_eq!(PairingState::Discovering.to_u8(), 1);
        assert_eq!(PairingState::Paired.to_u8(), 2);

        assert_eq!(PairingState::from_u8(0), PairingState::Idle);
        assert_eq!(PairingState::from_u8(1), PairingState::Discovering);
        assert_eq!(PairingState::from_u8(2), PairingState::Paired);
        assert_eq!(PairingState::from_u8(99), PairingState::Idle); // Fallback
    }

    #[test]
    fn test_uuid_format() {
        // Verify UUID format is valid (36 chars with hyphens)
        assert_eq!(HONEYLINK_SERVICE_UUID.len(), 36);
        assert_eq!(DEVICE_INFO_CHAR_UUID.len(), 36);
        assert_eq!(PAIRING_STATE_CHAR_UUID.len(), 36);

        // Verify hyphen positions (8-4-4-4-12)
        assert_eq!(HONEYLINK_SERVICE_UUID.chars().nth(8), Some('-'));
        assert_eq!(HONEYLINK_SERVICE_UUID.chars().nth(13), Some('-'));
        assert_eq!(HONEYLINK_SERVICE_UUID.chars().nth(18), Some('-'));
        assert_eq!(HONEYLINK_SERVICE_UUID.chars().nth(23), Some('-'));
    }

    #[test]
    fn test_max_gatt_value_size() {
        // Verify protocol constants
        assert_eq!(MAX_GATT_VALUE_SIZE, 20);

        let info = GattDeviceInfo::new("TEST", DeviceType::Desktop);
        assert_eq!(info.to_bytes().len(), MAX_GATT_VALUE_SIZE);

        let state = GattPairingState::new_with_random_nonce(PairingState::Idle);
        assert_eq!(state.to_bytes().len(), MAX_GATT_VALUE_SIZE);
    }
}
