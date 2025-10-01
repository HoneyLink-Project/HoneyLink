//! Idempotency key management
//!
//! Prevents duplicate request processing by tracking request hashes
//! and caching responses for 24 hours per spec/modules/session-orchestrator.md

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::{Error, Result};

/// FNV-1a hash for fast request fingerprinting
fn fnv1a_hash(data: &[u8]) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET_BASIS;
    for &byte in data {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Idempotency record with cached response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdempotencyRecord {
    /// Unique idempotency key (UUIDv4 or random 36-char string)
    pub idempotency_key: String,

    /// FNV-1a hash of request body for tamper detection
    pub request_hash: u64,

    /// Cached response snapshot (JSON)
    pub response_snapshot: serde_json::Value,

    /// Record creation timestamp
    pub created_at: DateTime<Utc>,

    /// Record expiration (24h retention)
    pub expires_at: DateTime<Utc>,
}

impl IdempotencyRecord {
    /// Create new idempotency record
    pub fn new(
        idempotency_key: String,
        request_body: &[u8],
        response: serde_json::Value,
    ) -> Self {
        let now = Utc::now();
        Self {
            idempotency_key,
            request_hash: fnv1a_hash(request_body),
            response_snapshot: response,
            created_at: now,
            expires_at: now + Duration::hours(24),
        }
    }

    /// Check if record has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Verify request hash matches (tamper detection)
    pub fn verify_request_hash(&self, request_body: &[u8]) -> bool {
        fnv1a_hash(request_body) == self.request_hash
    }
}

/// In-memory idempotency key store
///
/// NOTE: Production should use Redis or distributed cache for multi-instance deployments
pub struct IdempotencyStore {
    records: HashMap<String, IdempotencyRecord>,
}

impl IdempotencyStore {
    /// Create new empty store
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
        }
    }

    /// Store idempotency record
    ///
    /// # Errors
    /// Returns `Error::IdempotencyKeyExists` if key already exists and is not expired
    pub fn store(
        &mut self,
        idempotency_key: String,
        request_body: &[u8],
        response: serde_json::Value,
    ) -> Result<()> {
        // Check if key exists and is still valid
        if let Some(existing) = self.records.get(&idempotency_key) {
            if !existing.is_expired() {
                return Err(Error::IdempotencyKeyExists(idempotency_key));
            }
        }

        let record = IdempotencyRecord::new(idempotency_key.clone(), request_body, response);
        self.records.insert(idempotency_key, record);
        Ok(())
    }

    /// Get cached response for idempotency key
    ///
    /// Returns `Some((response, is_tampered))` if key exists and not expired
    /// `is_tampered` is true if request_hash mismatch detected
    pub fn get(
        &self,
        idempotency_key: &str,
        request_body: &[u8],
    ) -> Option<(serde_json::Value, bool)> {
        self.records.get(idempotency_key).and_then(|record| {
            if record.is_expired() {
                None
            } else {
                let is_tampered = !record.verify_request_hash(request_body);
                Some((record.response_snapshot.clone(), is_tampered))
            }
        })
    }

    /// Remove expired records (garbage collection)
    ///
    /// Returns number of records removed
    pub fn cleanup_expired(&mut self) -> usize {
        let initial_count = self.records.len();
        self.records.retain(|_, record| !record.is_expired());
        initial_count - self.records.len()
    }

    /// Get number of active records
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Check if store is empty
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

impl Default for IdempotencyStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_fnv1a_hash() {
        let data1 = b"hello world";
        let data2 = b"hello world";
        let data3 = b"hello worlD";

        assert_eq!(fnv1a_hash(data1), fnv1a_hash(data2));
        assert_ne!(fnv1a_hash(data1), fnv1a_hash(data3));
    }

    #[test]
    fn test_idempotency_record_creation() {
        let request = b"test request body";
        let response = json!({"status": "success"});

        let record = IdempotencyRecord::new(
            "test-key-123".to_string(),
            request,
            response.clone(),
        );

        assert_eq!(record.idempotency_key, "test-key-123");
        assert_eq!(record.response_snapshot, response);
        assert!(!record.is_expired());
        assert!(record.verify_request_hash(request));
    }

    #[test]
    fn test_idempotency_record_tamper_detection() {
        let request = b"original request";
        let response = json!({"status": "success"});

        let record = IdempotencyRecord::new(
            "test-key".to_string(),
            request,
            response,
        );

        assert!(record.verify_request_hash(request));
        assert!(!record.verify_request_hash(b"tampered request"));
    }

    #[test]
    fn test_idempotency_store_basic() {
        let mut store = IdempotencyStore::new();
        let request = b"test request";
        let response = json!({"session_id": "12345"});

        // Store new record
        assert!(store.store("key1".to_string(), request, response.clone()).is_ok());
        assert_eq!(store.len(), 1);

        // Retrieve record
        let result = store.get("key1", request);
        assert!(result.is_some());
        let (cached_response, is_tampered) = result.unwrap();
        assert_eq!(cached_response, response);
        assert!(!is_tampered);
    }

    #[test]
    fn test_idempotency_store_duplicate_key() {
        let mut store = IdempotencyStore::new();
        let request = b"test request";
        let response = json!({"status": "success"});

        // Store first record
        assert!(store.store("key1".to_string(), request, response.clone()).is_ok());

        // Attempt to store with same key should fail
        let result = store.store("key1".to_string(), request, response);
        assert!(result.is_err());
        match result {
            Err(Error::IdempotencyKeyExists(key)) => assert_eq!(key, "key1"),
            _ => panic!("Expected IdempotencyKeyExists error"),
        }
    }

    #[test]
    fn test_idempotency_store_tamper_detection() {
        let mut store = IdempotencyStore::new();
        let original_request = b"original request";
        let tampered_request = b"tampered request";
        let response = json!({"status": "success"});

        store.store("key1".to_string(), original_request, response).unwrap();

        // Retrieve with tampered request
        let result = store.get("key1", tampered_request);
        assert!(result.is_some());
        let (_, is_tampered) = result.unwrap();
        assert!(is_tampered, "Tamper detection should flag mismatch");
    }

    #[test]
    fn test_idempotency_store_expiration() {
        let mut store = IdempotencyStore::new();
        let request = b"test request";
        let response = json!({"status": "success"});

        store.store("key1".to_string(), request, response).unwrap();

        // Manually expire the record
        if let Some(record) = store.records.get_mut("key1") {
            record.expires_at = Utc::now() - Duration::hours(1);
        }

        // Should not retrieve expired record
        let result = store.get("key1", request);
        assert!(result.is_none());
    }

    #[test]
    fn test_idempotency_store_cleanup() {
        let mut store = IdempotencyStore::new();
        let request = b"test request";
        let response = json!({"status": "success"});

        // Store 3 records
        store.store("key1".to_string(), request, response.clone()).unwrap();
        store.store("key2".to_string(), request, response.clone()).unwrap();
        store.store("key3".to_string(), request, response).unwrap();
        assert_eq!(store.len(), 3);

        // Expire 2 records
        for key in ["key1", "key2"] {
            if let Some(record) = store.records.get_mut(key) {
                record.expires_at = Utc::now() - Duration::hours(1);
            }
        }

        // Cleanup should remove 2 records
        let removed = store.cleanup_expired();
        assert_eq!(removed, 2);
        assert_eq!(store.len(), 1);
    }
}
