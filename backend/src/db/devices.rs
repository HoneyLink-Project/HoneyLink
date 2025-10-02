// Device database operations

use crate::error::ApiError;
use crate::types::DeviceId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, types::Json};
use std::collections::HashMap;

/// Device status in the database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum DeviceStatus {
    Pending,
    Paired,
    Revoked,
}

impl std::fmt::Display for DeviceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceStatus::Pending => write!(f, "pending"),
            DeviceStatus::Paired => write!(f, "paired"),
            DeviceStatus::Revoked => write!(f, "revoked"),
        }
    }
}

/// Device record in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub device_id: String,
    pub public_key: Vec<u8>,
    pub firmware_version: String,
    pub capabilities: Vec<String>,
    pub attestation_format: Option<String>,
    pub attestation_evidence: Option<Vec<u8>>,
    pub attestation_nonce: Option<String>,
    pub metadata: Option<Json<HashMap<String, serde_json::Value>>>,
    pub device_token: String,
    pub status: String,
    pub certificate_serial: Option<String>,
    pub registered_at: DateTime<Utc>,
    pub paired_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

/// Create device parameters
#[derive(Debug, Clone)]
pub struct CreateDeviceParams {
    pub device_id: DeviceId,
    pub public_key: Vec<u8>,
    pub firmware_version: String,
    pub capabilities: Vec<String>,
    pub attestation_format: Option<String>,
    pub attestation_evidence: Option<Vec<u8>>,
    pub attestation_nonce: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub device_token: String,
}

/// Creates a new device record
pub async fn create_device(pool: &PgPool, params: CreateDeviceParams) -> Result<Device, ApiError> {
    let metadata_json = params.metadata.map(Json);

    let device = sqlx::query_as!(
        Device,
        r#"
        INSERT INTO devices (
            device_id, public_key, firmware_version, capabilities,
            attestation_format, attestation_evidence, attestation_nonce,
            metadata, device_token, status
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING
            device_id, public_key, firmware_version, capabilities,
            attestation_format, attestation_evidence, attestation_nonce,
            metadata as "metadata: Json<HashMap<String, serde_json::Value>>",
            device_token, status, certificate_serial,
            registered_at, paired_at, updated_at
        "#,
        params.device_id.0,
        params.public_key,
        params.firmware_version,
        &params.capabilities,
        params.attestation_format,
        params.attestation_evidence,
        params.attestation_nonce,
        metadata_json as Option<Json<HashMap<String, serde_json::Value>>>,
        params.device_token,
        DeviceStatus::Pending.to_string()
    )
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(db_err) if db_err.constraint() == Some("devices_pkey") => {
            ApiError::Conflict("Device ID already exists".to_string())
        }
        sqlx::Error::Database(db_err) if db_err.constraint() == Some("devices_device_token_key") => {
            ApiError::Conflict("Device token already exists".to_string())
        }
        _ => ApiError::Dependency(format!("Database error: {}", e)),
    })?;

    Ok(device)
}

/// Gets a device by ID
pub async fn get_device(pool: &PgPool, device_id: &DeviceId) -> Result<Option<Device>, ApiError> {
    let device = sqlx::query_as!(
        Device,
        r#"
        SELECT
            device_id, public_key, firmware_version, capabilities,
            attestation_format, attestation_evidence, attestation_nonce,
            metadata as "metadata: Json<HashMap<String, serde_json::Value>>",
            device_token, status, certificate_serial,
            registered_at, paired_at, updated_at
        FROM devices
        WHERE device_id = $1
        "#,
        device_id.0
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| ApiError::Dependency(format!("Database error: {}", e)))?;

    Ok(device)
}

/// Updates device status to paired
pub async fn mark_device_paired(
    pool: &PgPool,
    device_id: &DeviceId,
    certificate_serial: &str,
) -> Result<(), ApiError> {
    let result = sqlx::query!(
        r#"
        UPDATE devices
        SET status = $1, paired_at = NOW(), certificate_serial = $2
        WHERE device_id = $3 AND status = $4
        "#,
        DeviceStatus::Paired.to_string(),
        certificate_serial,
        device_id.0,
        DeviceStatus::Pending.to_string()
    )
    .execute(pool)
    .await
    .map_err(|e| ApiError::Dependency(format!("Database error: {}", e)))?;

    if result.rows_affected() == 0 {
        return Err(ApiError::State("Device is not in pending state or does not exist".to_string()));
    }

    Ok(())
}

/// Checks if a device exists
pub async fn device_exists(pool: &PgPool, device_id: &DeviceId) -> Result<bool, ApiError> {
    let exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM devices WHERE device_id = $1)",
        device_id.0
    )
    .fetch_one(pool)
    .await
    .map_err(|e| ApiError::Dependency(format!("Database error: {}", e)))?;

    Ok(exists.unwrap_or(false))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_status_display() {
        assert_eq!(DeviceStatus::Pending.to_string(), "pending");
        assert_eq!(DeviceStatus::Paired.to_string(), "paired");
        assert_eq!(DeviceStatus::Revoked.to_string(), "revoked");
    }
}
