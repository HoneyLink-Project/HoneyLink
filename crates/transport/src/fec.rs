//! Forward Error Correction (FEC) strategies
//!
//! This module implements Reed-Solomon based FEC encoding/decoding with three modes:
//! - **None**: No redundancy (0% overhead)
//! - **Light**: 20% redundancy for mild packet loss (5-10%)
//! - **Heavy**: 50% redundancy for severe packet loss (>10%)
//!
//! # Design Rationale
//! - Pure Rust implementation (reed-solomon-erasure crate, no C/C++ dependencies)
//! - Dynamic mode switching based on observed packet loss rate
//! - CRC32 checksums for data integrity verification
//! - Performance target: P95 < 5ms for encoding (MOD-003 spec)

use crate::TransportError;
use reed_solomon_erasure::galois_8::ReedSolomon;

/// FEC strategy modes with different redundancy levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FecStrategy {
    /// No FEC encoding (0% overhead)
    None,
    /// Light FEC encoding (20% redundancy, 4/5 rate)
    Light,
    /// Heavy FEC encoding (50% redundancy, 2/3 rate)
    Heavy,
}

impl FecStrategy {
    /// Returns the overhead percentage for this strategy
    pub fn overhead_percent(&self) -> u8 {
        match self {
            Self::None => 0,
            Self::Light => 20,
            Self::Heavy => 50,
        }
    }

    /// Returns the number of data shards for a given total shard count
    ///
    /// # Formula
    /// - None: data_shards = total_shards
    /// - Light: data_shards = total_shards * 5/6 (20% parity)
    /// - Heavy: data_shards = total_shards * 2/3 (50% parity)
    pub fn data_shards(&self, total_shards: usize) -> usize {
        match self {
            Self::None => total_shards,
            Self::Light => (total_shards * 5) / 6,
            Self::Heavy => (total_shards * 2) / 3,
        }
    }

    /// Returns the number of parity shards for a given total shard count
    pub fn parity_shards(&self, total_shards: usize) -> usize {
        match self {
            Self::None => 0,
            Self::Light => total_shards - self.data_shards(total_shards),
            Self::Heavy => total_shards - self.data_shards(total_shards),
        }
    }

    /// Selects FEC strategy based on observed packet loss rate
    ///
    /// # Selection Logic (MOD-003 spec)
    /// - loss_rate < 5%: None
    /// - 5% <= loss_rate < 10%: Light
    /// - loss_rate >= 10%: Heavy
    pub fn select_for_loss_rate(loss_rate: f32) -> Self {
        if loss_rate < 0.05 {
            Self::None
        } else if loss_rate < 0.10 {
            Self::Light
        } else {
            Self::Heavy
        }
    }
}

/// FEC encoder/decoder with Reed-Solomon implementation
pub struct FecEncoder {
    strategy: FecStrategy,
    /// Shard size in bytes (default: 1024)
    shard_size: usize,
}

impl FecEncoder {
    /// Creates a new FEC encoder with the specified strategy
    ///
    /// # Arguments
    /// * `strategy` - FEC strategy to use
    /// * `shard_size` - Size of each shard in bytes (default: 1024)
    pub fn new(strategy: FecStrategy, shard_size: usize) -> Self {
        Self {
            strategy,
            shard_size,
        }
    }

    /// Returns the current FEC strategy
    pub fn strategy(&self) -> FecStrategy {
        self.strategy
    }

    /// Changes the FEC strategy (dynamic mode switching)
    pub fn set_strategy(&mut self, strategy: FecStrategy) {
        self.strategy = strategy;
    }

    /// Encodes data with FEC redundancy
    ///
    /// # Arguments
    /// * `data` - Raw data to encode
    ///
    /// # Returns
    /// * `Ok(Vec<Vec<u8>>)` - Vector of shards (data + parity)
    /// * `Err(TransportError::FecDecodingFailed)` on encoding failure
    ///
    /// # Performance
    /// * Target: P95 < 5ms (MOD-003 spec)
    pub fn encode(&self, data: &[u8]) -> Result<Vec<Vec<u8>>, TransportError> {
        match self.strategy {
            FecStrategy::None => {
                // No FEC: return data as single shard with CRC32 checksum
                let mut shard = data.to_vec();
                let checksum = crc32fast::hash(data);
                shard.extend_from_slice(&checksum.to_le_bytes());
                Ok(vec![shard])
            }
            _ => {
                // Calculate shard counts
                let data_with_crc = add_crc32(data);
                let num_shards = (data_with_crc.len() + self.shard_size - 1) / self.shard_size;
                let num_shards = num_shards.max(6); // Minimum 6 shards for meaningful RS coding
                let data_shards = self.strategy.data_shards(num_shards);
                let parity_shards = self.strategy.parity_shards(num_shards);

                // Pad data to fit evenly into data shards
                let padded_len = data_shards * self.shard_size;
                let mut padded_data = data_with_crc;
                padded_data.resize(padded_len, 0);

                // Split into shards
                let mut shards: Vec<Vec<u8>> = padded_data
                    .chunks(self.shard_size)
                    .map(|chunk| chunk.to_vec())
                    .collect();

                // Add empty parity shards
                shards.resize(data_shards + parity_shards, vec![0u8; self.shard_size]);

                // Create Reed-Solomon encoder
                let rs = ReedSolomon::new(data_shards, parity_shards).map_err(|e| {
                    TransportError::FecDecodingFailed(format!("RS creation failed: {}", e))
                })?;

                // Encode parity shards
                rs.encode(&mut shards).map_err(|e| {
                    TransportError::FecDecodingFailed(format!("RS encoding failed: {}", e))
                })?;

                Ok(shards)
            }
        }
    }

    /// Decodes FEC-encoded shards back to original data
    ///
    /// # Arguments
    /// * `shards` - Vector of shards (some may be None if lost)
    /// * `original_len` - Original data length (before padding)
    ///
    /// # Returns
    /// * `Ok(Vec<u8>)` - Recovered data
    /// * `Err(TransportError::FecDecodingFailed)` if too many shards are lost
    pub fn decode(
        &self,
        shards: &[Option<Vec<u8>>],
        original_len: usize,
    ) -> Result<Vec<u8>, TransportError> {
        match self.strategy {
            FecStrategy::None => {
                // No FEC: extract data and verify CRC32
                let shard = shards
                    .first()
                    .and_then(|s| s.as_ref())
                    .ok_or_else(|| TransportError::FecDecodingFailed("No shard available".into()))?;

                if shard.len() < 4 {
                    return Err(TransportError::FecDecodingFailed("Shard too small".into()));
                }

                let data = &shard[..shard.len() - 4];
                let stored_crc = u32::from_le_bytes([
                    shard[shard.len() - 4],
                    shard[shard.len() - 3],
                    shard[shard.len() - 2],
                    shard[shard.len() - 1],
                ]);
                let computed_crc = crc32fast::hash(data);

                if stored_crc != computed_crc {
                    return Err(TransportError::FecDecodingFailed("CRC mismatch".into()));
                }

                Ok(data[..original_len.min(data.len())].to_vec())
            }
            _ => {
                // Calculate shard configuration
                let num_shards = shards.len();
                let data_shards = self.strategy.data_shards(num_shards);
                let parity_shards = self.strategy.parity_shards(num_shards);

                // Convert to mutable Vec for RS decoder
                let mut shard_vec: Vec<Option<Vec<u8>>> = shards.to_vec();

                // Create Reed-Solomon decoder
                let rs = ReedSolomon::new(data_shards, parity_shards).map_err(|e| {
                    TransportError::FecDecodingFailed(format!("RS creation failed: {}", e))
                })?;

                // Decode (reconstruct missing shards)
                rs.reconstruct(&mut shard_vec).map_err(|e| {
                    TransportError::FecDecodingFailed(format!("RS decoding failed: {}", e))
                })?;

                // Concatenate data shards
                let mut data = Vec::with_capacity(data_shards * self.shard_size);
                for shard_opt in shard_vec.iter().take(data_shards) {
                    if let Some(shard) = shard_opt {
                        data.extend_from_slice(shard);
                    }
                }

                // Remove padding and verify CRC32
                if data.len() < 4 {
                    return Err(TransportError::FecDecodingFailed("Data too small".into()));
                }

                let stored_crc = u32::from_le_bytes([
                    data[data.len() - 4],
                    data[data.len() - 3],
                    data[data.len() - 2],
                    data[data.len() - 1],
                ]);
                data.truncate(data.len() - 4);
                let computed_crc = crc32fast::hash(&data);

                if stored_crc != computed_crc {
                    return Err(TransportError::FecDecodingFailed("CRC mismatch".into()));
                }

                data.truncate(original_len);
                Ok(data)
            }
        }
    }
}

impl Default for FecEncoder {
    fn default() -> Self {
        Self::new(FecStrategy::None, 1024)
    }
}

/// Adds CRC32 checksum to data
fn add_crc32(data: &[u8]) -> Vec<u8> {
    let mut result = data.to_vec();
    let checksum = crc32fast::hash(data);
    result.extend_from_slice(&checksum.to_le_bytes());
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fec_strategy_overhead() {
        assert_eq!(FecStrategy::None.overhead_percent(), 0);
        assert_eq!(FecStrategy::Light.overhead_percent(), 20);
        assert_eq!(FecStrategy::Heavy.overhead_percent(), 50);
    }

    #[test]
    fn test_fec_strategy_selection() {
        assert_eq!(FecStrategy::select_for_loss_rate(0.01), FecStrategy::None);
        assert_eq!(FecStrategy::select_for_loss_rate(0.07), FecStrategy::Light);
        assert_eq!(FecStrategy::select_for_loss_rate(0.15), FecStrategy::Heavy);
    }

    #[test]
    fn test_fec_encode_decode_none() {
        let encoder = FecEncoder::new(FecStrategy::None, 1024);
        let data = b"Hello, World!";

        let shards = encoder.encode(data).unwrap();
        assert_eq!(shards.len(), 1);

        let decoded = encoder
            .decode(&[Some(shards[0].clone())], data.len())
            .unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_fec_encode_decode_light() {
        let encoder = FecEncoder::new(FecStrategy::Light, 128);
        let data = vec![42u8; 600];

        let shards = encoder.encode(&data).unwrap();
        assert!(shards.len() >= 6);

        // Simulate losing one shard
        let mut received_shards: Vec<Option<Vec<u8>>> =
            shards.into_iter().map(Some).collect();
        received_shards[2] = None;

        let decoded = encoder.decode(&received_shards, data.len()).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_crc_mismatch_detection() {
        let encoder = FecEncoder::new(FecStrategy::None, 1024);
        let data = b"Test data";

        let mut shards = encoder.encode(data).unwrap();
        // Corrupt the data
        shards[0][0] ^= 0xFF;

        let result = encoder.decode(&[Some(shards[0].clone())], data.len());
        assert!(matches!(result, Err(TransportError::FecDecodingFailed(_))));
    }
}
