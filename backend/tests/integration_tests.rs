// Integration tests for device management API

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    // Note: These tests require a PostgreSQL/CockroachDB instance
    // Set DATABASE_URL environment variable before running
    // They are marked with #[ignore] to prevent CI/CD failures without DB setup

    async fn setup_test_db() -> PgPool {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://localhost/honeylink_test".to_string());

        let pool = sqlx::PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        pool
    }

    #[tokio::test]
    #[ignore = "Requires PostgreSQL database"]
    async fn test_create_device() {
        let pool = setup_test_db().await;

        let params = crate::db::devices::CreateDeviceParams {
            device_id: crate::types::DeviceId("TEST-DEVICE-001".to_string()),
            public_key: vec![1u8; 32],
            firmware_version: "1.0.0".to_string(),
            capabilities: vec!["telemetry".to_string()],
            attestation_format: None,
            attestation_evidence: None,
            attestation_nonce: None,
            metadata: None,
            device_token: "test-token-123".to_string(),
        };

        let device = crate::db::devices::create_device(&pool, params).await;
        assert!(device.is_ok());

        let device = device.unwrap();
        assert_eq!(device.device_id, "TEST-DEVICE-001");
        assert_eq!(device.status, "pending");
    }

    #[tokio::test]
    #[ignore = "Requires PostgreSQL database"]
    async fn test_create_pairing_code() {
        let pool = setup_test_db().await;

        // Create device first
        let params = crate::db::devices::CreateDeviceParams {
            device_id: crate::types::DeviceId("TEST-DEVICE-002".to_string()),
            public_key: vec![2u8; 32],
            firmware_version: "1.0.0".to_string(),
            capabilities: vec!["telemetry".to_string()],
            attestation_format: None,
            attestation_evidence: None,
            attestation_nonce: None,
            metadata: None,
            device_token: "test-token-456".to_string(),
        };
        crate::db::devices::create_device(&pool, params).await.unwrap();

        // Create pairing code
        let device_id = crate::types::DeviceId("TEST-DEVICE-002".to_string());
        let result = crate::db::pairing::create_pairing_code(&pool, &device_id).await;

        assert!(result.is_ok());
        let (code, expires_at) = result.unwrap();
        assert_eq!(code.len(), 14); // "XXXX-XXXX-XXXX"
        assert!(expires_at > chrono::Utc::now());
    }

    #[tokio::test]
    #[ignore = "Requires PostgreSQL database"]
    async fn test_record_audit_event() {
        let pool = setup_test_db().await;

        let params = crate::db::audit::CreateAuditEventParams {
            category: crate::db::audit::AuditCategory::DeviceRegistration,
            actor: "test-user".to_string(),
            device_id: Some("TEST-DEVICE-003".to_string()),
            outcome: crate::db::audit::AuditOutcome::Success,
            details: Some(serde_json::json!({"test": "data"})),
            trace_id: Some("00-test-trace-id-00".to_string()),
        };

        let event_id = crate::db::audit::record_audit_event(&pool, params).await;
        assert!(event_id.is_ok());
    }

    #[test]
    fn test_pairing_code_format() {
        let code = crate::db::pairing::generate_pairing_code();

        // Check format "XXXX-XXXX-XXXX"
        assert_eq!(code.len(), 14);
        assert_eq!(code.chars().nth(4).unwrap(), '-');
        assert_eq!(code.chars().nth(9).unwrap(), '-');

        // Check no ambiguous characters
        assert!(!code.contains('0'));
        assert!(!code.contains('O'));
        assert!(!code.contains('1'));
        assert!(!code.contains('I'));
        assert!(!code.contains('l'));
    }

    // ========== Session Management Tests ==========

    #[tokio::test]
    #[ignore = "Requires PostgreSQL database"]
    async fn test_create_session() {
        let pool = setup_test_db().await;

        // First create a paired device
        let device_params = crate::db::devices::CreateDeviceParams {
            device_id: crate::types::DeviceId("TEST-DEVICE-SESSION-001".to_string()),
            public_key: vec![42u8; 32], // Dummy key
            firmware_version: "1.0.0".to_string(),
            capabilities: vec!["telemetry".to_string()],
            attestation_format: None,
            attestation_evidence: None,
            attestation_nonce: None,
            metadata: None,
            device_token: "test-token-session-001".to_string(),
        };
        crate::db::devices::create_device(&pool, device_params).await.unwrap();

        // Mark device as paired
        crate::db::devices::mark_device_paired(
            &pool,
            &crate::types::DeviceId("TEST-DEVICE-SESSION-001".to_string()),
            "serial-12345",
        )
        .await
        .unwrap();

        // Create session
        let session_id = uuid::Uuid::now_v7();
        let streams_json = serde_json::json!([
            {
                "stream_id": uuid::Uuid::now_v7(),
                "name": "telemetry",
                "connection_id": "conn-001",
                "fec": {"data_shards": 10, "parity_shards": 2}
            }
        ]);

        let session_params = crate::db::sessions::CreateSessionParams {
            session_id,
            device_id: "TEST-DEVICE-SESSION-001".to_string(),
            streams: streams_json,
            key_material: vec![1u8; 32],
            ttl_seconds: 3600,
            endpoint: "quic://127.0.0.1:7843".to_string(),
        };

        let session = crate::db::sessions::create_session(&pool, session_params).await;
        assert!(session.is_ok());

        let session = session.unwrap();
        assert_eq!(session.session_id, session_id);
        assert_eq!(session.device_id, "TEST-DEVICE-SESSION-001");
        assert_eq!(session.status, crate::db::sessions::SessionStatus::Active);
    }

    #[tokio::test]
    #[ignore = "Requires PostgreSQL database"]
    async fn test_get_device_sessions() {
        let pool = setup_test_db().await;

        // Create device
        let device_params = crate::db::devices::CreateDeviceParams {
            device_id: crate::types::DeviceId("TEST-DEVICE-SESSION-002".to_string()),
            public_key: vec![42u8; 32],
            firmware_version: "1.0.0".to_string(),
            capabilities: vec!["telemetry".to_string()],
            attestation_format: None,
            attestation_evidence: None,
            attestation_nonce: None,
            metadata: None,
            device_token: "test-token-session-002".to_string(),
        };
        crate::db::devices::create_device(&pool, device_params).await.unwrap();

        crate::db::devices::mark_device_paired(
            &pool,
            &crate::types::DeviceId("TEST-DEVICE-SESSION-002".to_string()),
            "serial-12346",
        )
        .await
        .unwrap();

        // Create two sessions
        for i in 0..2 {
            let session_params = crate::db::sessions::CreateSessionParams {
                session_id: uuid::Uuid::now_v7(),
                device_id: "TEST-DEVICE-SESSION-002".to_string(),
                streams: serde_json::json!([]),
                key_material: vec![i; 32],
                ttl_seconds: 3600,
                endpoint: "quic://127.0.0.1:7843".to_string(),
            };
            crate::db::sessions::create_session(&pool, session_params).await.unwrap();
        }

        // Retrieve sessions
        let sessions = crate::db::sessions::get_device_sessions(&pool, "TEST-DEVICE-SESSION-002").await;
        assert!(sessions.is_ok());
        assert_eq!(sessions.unwrap().len(), 2);
    }

    #[tokio::test]
    #[ignore = "Requires PostgreSQL database"]
    async fn test_terminate_session() {
        let pool = setup_test_db().await;

        // Create device
        let device_params = crate::db::devices::CreateDeviceParams {
            device_id: crate::types::DeviceId("TEST-DEVICE-SESSION-003".to_string()),
            public_key: vec![42u8; 32],
            firmware_version: "1.0.0".to_string(),
            capabilities: vec!["telemetry".to_string()],
            attestation_format: None,
            attestation_evidence: None,
            attestation_nonce: None,
            metadata: None,
            device_token: "test-token-session-003".to_string(),
        };
        crate::db::devices::create_device(&pool, device_params).await.unwrap();

        crate::db::devices::mark_device_paired(
            &pool,
            &crate::types::DeviceId("TEST-DEVICE-SESSION-003".to_string()),
            "serial-12347",
        )
        .await
        .unwrap();

        // Create session
        let session_id = uuid::Uuid::now_v7();
        let session_params = crate::db::sessions::CreateSessionParams {
            session_id,
            device_id: "TEST-DEVICE-SESSION-003".to_string(),
            streams: serde_json::json!([]),
            key_material: vec![1u8; 32],
            ttl_seconds: 3600,
            endpoint: "quic://127.0.0.1:7843".to_string(),
        };
        crate::db::sessions::create_session(&pool, session_params).await.unwrap();

        // Terminate session
        let result = crate::db::sessions::terminate_session(&pool, session_id).await;
        assert!(result.is_ok());

        // Verify status changed
        let session = crate::db::sessions::get_session(&pool, session_id).await.unwrap();
        assert_eq!(session.status, crate::db::sessions::SessionStatus::Terminated);
        assert!(session.terminated_at.is_some());
    }

    #[test]
    fn test_session_status_serialization() {
        let status = crate::db::sessions::SessionStatus::Active;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, r#""active""#);

        let deserialized: crate::db::sessions::SessionStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, crate::db::sessions::SessionStatus::Active);
    }

    #[test]
    fn test_fec_params_calculation() {
        use crate::routes::sessions::FecParams;
        use honeylink_qos_scheduler::scheduler::QoSPriority;

        let burst = FecParams::from_priority(QoSPriority::Burst);
        assert_eq!(burst.data_shards, 10);
        assert_eq!(burst.parity_shards, 5);

        let normal = FecParams::from_priority(QoSPriority::Normal);
        assert_eq!(normal.parity_shards, 2);

        let latency = FecParams::from_priority(QoSPriority::Latency);
        assert_eq!(latency.parity_shards, 1);
    }
}
