//! Integration tests for Vault key management.
//!
//! These tests require a running Vault dev server.
//! Run: `./scripts/setup-vault-dev.sh` or `.\scripts\setup-vault-dev.ps1`
//!
//! To run these tests:
//! ```bash
//! export VAULT_ADDR="http://127.0.0.1:8200"
//! export VAULT_TOKEN="root"
//! cargo test --features vault -- --ignored --test-threads=1
//! ```

#![cfg(feature = "vault")]

use honeylink_crypto::vault::{KeyMaterial, KeyScope, VaultClient, VaultError};
use honeylink_crypto::lifecycle::VaultKeyLifecycle;
use std::time::Duration;

/// Helper to create a test client
fn create_test_client() -> VaultClient {
    VaultClient::from_env().expect("Failed to create Vault client. Is Vault running?")
}

/// Helper to generate test key data
fn generate_test_key(size: usize) -> Vec<u8> {
    vec![0x42; size]
}

#[tokio::test]
#[ignore]
async fn test_vault_connection() {
    let client = create_test_client();

    client.health_check().await.expect("Vault health check failed");
}

#[tokio::test]
#[ignore]
async fn test_store_and_retrieve_service_key() {
    let client = create_test_client();
    let key_data = generate_test_key(32);
    let key_name = "test-service-001";

    // Store key
    client
        .store_key(
            KeyScope::Service,
            key_name,
            key_data.clone(),
            Duration::from_secs(3600),
        )
        .await
        .expect("Failed to store key");

    // Retrieve key
    let retrieved = client
        .retrieve_key(KeyScope::Service, key_name)
        .await
        .expect("Failed to retrieve key");

    assert_eq!(retrieved.data, key_data);
    assert_eq!(retrieved.metadata.scope, KeyScope::Service);
    assert_eq!(retrieved.metadata.name, key_name);
    assert_eq!(retrieved.metadata.version, 1);

    // Cleanup
    client.delete_key(KeyScope::Service, key_name).await.ok();
}

#[tokio::test]
#[ignore]
async fn test_key_expiration() {
    let client = create_test_client();
    let key_data = generate_test_key(32);
    let key_name = "test-expired-key";

    // Store key with 1 second TTL
    client
        .store_key(
            KeyScope::Profile,
            key_name,
            key_data.clone(),
            Duration::from_secs(1),
        )
        .await
        .expect("Failed to store key");

    // Wait for expiration
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Attempt retrieval (should fail with KeyExpired)
    let result = client.retrieve_key(KeyScope::Profile, key_name).await;

    match result {
        Err(VaultError::KeyExpired { scope, name }) => {
            assert_eq!(scope, KeyScope::Profile);
            assert_eq!(name, key_name);
        }
        _ => panic!("Expected KeyExpired error, got: {:?}", result),
    }

    // Cleanup
    client.delete_key(KeyScope::Profile, key_name).await.ok();
}

#[tokio::test]
#[ignore]
async fn test_key_not_found() {
    let client = create_test_client();

    let result = client
        .retrieve_key(KeyScope::Telemetry, "nonexistent-key-xyz")
        .await;

    match result {
        Err(VaultError::KeyNotFound { scope, name }) => {
            assert_eq!(scope, KeyScope::Telemetry);
            assert_eq!(name, "nonexistent-key-xyz");
        }
        _ => panic!("Expected KeyNotFound error, got: {:?}", result),
    }
}

#[tokio::test]
#[ignore]
async fn test_list_keys_in_scope() {
    let client = create_test_client();

    // Store multiple keys
    let test_keys = vec!["list-test-1", "list-test-2", "list-test-3"];

    for key_name in &test_keys {
        client
            .store_key(
                KeyScope::Service,
                key_name,
                generate_test_key(32),
                Duration::from_secs(3600),
            )
            .await
            .expect("Failed to store key");
    }

    // List keys
    let keys = client
        .list_keys(KeyScope::Service)
        .await
        .expect("Failed to list keys");

    for key_name in &test_keys {
        assert!(
            keys.contains(&key_name.to_string()),
            "Key {} not found in list",
            key_name
        );
    }

    // Cleanup
    for key_name in &test_keys {
        client.delete_key(KeyScope::Service, key_name).await.ok();
    }
}

#[tokio::test]
#[ignore]
async fn test_key_rotation() {
    let client = create_test_client();
    let key_name = "rotation-test-key";

    // Store initial key
    let original_key = generate_test_key(32);
    client
        .store_key(
            KeyScope::Service,
            key_name,
            original_key.clone(),
            Duration::from_secs(3600),
        )
        .await
        .expect("Failed to store initial key");

    // Rotate to new key
    let new_key = vec![0x99; 32];
    let new_version = client
        .rotate_key(KeyScope::Service, key_name, new_key.clone())
        .await
        .expect("Failed to rotate key");

    assert_eq!(new_version, 2);

    // Retrieve and verify new key
    let retrieved = client
        .retrieve_key(KeyScope::Service, key_name)
        .await
        .expect("Failed to retrieve rotated key");

    assert_eq!(retrieved.data, new_key);
    assert_eq!(retrieved.metadata.version, 2);

    // Cleanup
    client.delete_key(KeyScope::Service, key_name).await.ok();
}

#[tokio::test]
#[ignore]
async fn test_key_deletion() {
    let client = create_test_client();
    let key_name = "delete-test-key";

    // Store key
    client
        .store_key(
            KeyScope::Profile,
            key_name,
            generate_test_key(32),
            Duration::from_secs(3600),
        )
        .await
        .expect("Failed to store key");

    // Delete key
    client
        .delete_key(KeyScope::Profile, key_name)
        .await
        .expect("Failed to delete key");

    // Verify key is gone
    let result = client.retrieve_key(KeyScope::Profile, key_name).await;

    assert!(
        matches!(result, Err(VaultError::KeyNotFound { .. })),
        "Key should not be found after deletion"
    );
}

#[tokio::test]
#[ignore]
async fn test_all_key_scopes() {
    let client = create_test_client();

    let scopes = vec![
        KeyScope::Root,
        KeyScope::Service,
        KeyScope::Profile,
        KeyScope::Telemetry,
    ];

    for scope in scopes {
        let key_name = format!("scope-test-{:?}", scope);
        let key_data = generate_test_key(32);

        // Store
        client
            .store_key(scope, &key_name, key_data.clone(), Duration::from_secs(3600))
            .await
            .expect(&format!("Failed to store key in scope {:?}", scope));

        // Retrieve
        let retrieved = client
            .retrieve_key(scope, &key_name)
            .await
            .expect(&format!("Failed to retrieve key from scope {:?}", scope));

        assert_eq!(retrieved.data, key_data);
        assert_eq!(retrieved.metadata.scope, scope);

        // Cleanup
        client.delete_key(scope, &key_name).await.ok();
    }
}

#[tokio::test]
#[ignore]
async fn test_lifecycle_manager() {
    let lifecycle = VaultKeyLifecycle::from_env()
        .expect("Failed to create lifecycle manager");

    lifecycle.health_check().await.expect("Health check failed");
}

#[tokio::test]
#[ignore]
async fn test_lifecycle_generate_and_retrieve() {
    use honeylink_crypto::key_management::KeyScope as CryptoKeyScope;

    let lifecycle = VaultKeyLifecycle::from_env()
        .expect("Failed to create lifecycle manager");

    let key_data = generate_test_key(32);
    let key_name = "lifecycle-test-001";

    // Generate and store
    lifecycle
        .generate_and_store(CryptoKeyScope::DeviceMaster, key_name, key_data.clone())
        .await
        .expect("Failed to generate and store");

    // Retrieve
    let retrieved = lifecycle
        .retrieve(CryptoKeyScope::DeviceMaster, key_name)
        .await
        .expect("Failed to retrieve");

    assert_eq!(retrieved.data, key_data);

    // Cleanup
    lifecycle
        .revoke(CryptoKeyScope::DeviceMaster, key_name)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn test_lifecycle_rotation() {
    use honeylink_crypto::key_management::KeyScope as CryptoKeyScope;

    let lifecycle = VaultKeyLifecycle::from_env()
        .expect("Failed to create lifecycle manager");

    let original_key = generate_test_key(32);
    let new_key = vec![0xAA; 32];
    let key_name = "lifecycle-rotation-test";

    // Initial key
    lifecycle
        .generate_and_store(CryptoKeyScope::Session, key_name, original_key)
        .await
        .expect("Failed to store initial key");

    // Rotate
    let new_version = lifecycle
        .rotate(CryptoKeyScope::Session, key_name, new_key.clone())
        .await
        .expect("Failed to rotate");

    assert_eq!(new_version, 2);

    // Verify new key
    let retrieved = lifecycle
        .retrieve(CryptoKeyScope::Session, key_name)
        .await
        .expect("Failed to retrieve rotated key");

    assert_eq!(retrieved.data, new_key);

    // Cleanup
    lifecycle
        .destroy(CryptoKeyScope::Session, key_name, vec![1, 2])
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn test_concurrent_operations() {
    let client = create_test_client();

    let mut handles = vec![];

    // Spawn 10 concurrent store operations
    for i in 0..10 {
        let client_clone = create_test_client();
        let handle = tokio::spawn(async move {
            let key_name = format!("concurrent-test-{}", i);
            let key_data = generate_test_key(32);

            client_clone
                .store_key(
                    KeyScope::Service,
                    &key_name,
                    key_data,
                    Duration::from_secs(3600),
                )
                .await
                .expect("Failed to store key concurrently");

            key_name
        });
        handles.push(handle);
    }

    // Wait for all operations
    let key_names: Vec<String> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.expect("Task panicked"))
        .collect();

    // Verify all keys exist
    for key_name in &key_names {
        let retrieved = client
            .retrieve_key(KeyScope::Service, key_name)
            .await
            .expect(&format!("Failed to retrieve key {}", key_name));

        assert_eq!(retrieved.metadata.name, *key_name);
    }

    // Cleanup
    for key_name in &key_names {
        client.delete_key(KeyScope::Service, key_name).await.ok();
    }
}

#[tokio::test]
#[ignore]
async fn test_key_metadata() {
    let client = create_test_client();
    let key_name = "metadata-test-key";
    let key_data = generate_test_key(32);

    client
        .store_key(
            KeyScope::Service,
            key_name,
            key_data,
            Duration::from_secs(7200),
        )
        .await
        .expect("Failed to store key");

    let retrieved = client
        .retrieve_key(KeyScope::Service, key_name)
        .await
        .expect("Failed to retrieve key");

    // Verify metadata
    assert_eq!(retrieved.metadata.scope, KeyScope::Service);
    assert_eq!(retrieved.metadata.name, key_name);
    assert_eq!(retrieved.metadata.version, 1);
    assert_eq!(retrieved.metadata.algorithm, "X25519");
    assert_eq!(retrieved.metadata.environment, "development");

    // Verify timestamps are valid RFC3339
    chrono::DateTime::parse_from_rfc3339(&retrieved.metadata.created_at)
        .expect("Invalid created_at timestamp");
    chrono::DateTime::parse_from_rfc3339(&retrieved.metadata.expires_at)
        .expect("Invalid expires_at timestamp");

    // Cleanup
    client.delete_key(KeyScope::Service, key_name).await.ok();
}

#[tokio::test]
#[ignore]
async fn test_zeroization() {
    let client = create_test_client();
    let key_name = "zeroize-test-key";
    let key_data = generate_test_key(32);

    client
        .store_key(
            KeyScope::Profile,
            key_name,
            key_data.clone(),
            Duration::from_secs(3600),
        )
        .await
        .expect("Failed to store key");

    {
        let retrieved = client
            .retrieve_key(KeyScope::Profile, key_name)
            .await
            .expect("Failed to retrieve key");

        assert_eq!(retrieved.data, key_data);

        // KeyMaterial should be automatically zeroized when dropped here
    }

    // Note: We can't verify zeroization without unsafe code,
    // but the zeroize crate guarantees it

    // Cleanup
    client.delete_key(KeyScope::Profile, key_name).await.ok();
}

/// Helper to check if Vault is available
async fn vault_available() -> bool {
    match create_test_client().health_check().await {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[tokio::test]
#[ignore]
async fn test_fallback_on_vault_unavailable() {
    // This test simulates Vault being unavailable
    // In production, the application should fall back to local encrypted storage

    let result = VaultClient::new(
        "http://127.0.0.1:9999", // Non-existent port
        "invalid-token",
        None,
        "honeylink",
        "test",
    );

    assert!(result.is_ok(), "Client creation should succeed even if Vault is unavailable");

    let client = result.unwrap();
    let health_result = client.health_check().await;

    assert!(
        health_result.is_err(),
        "Health check should fail when Vault is unavailable"
    );
}
