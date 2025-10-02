// Device management API route handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use base64::Engine;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::collections::HashMap;

use crate::{
    db::{
        audit::{record_audit_event, AuditCategory, AuditOutcome, CreateAuditEventParams},
        devices::{create_device, get_device, mark_device_paired, CreateDeviceParams},
        pairing::{create_pairing_code, validate_and_consume_pairing_code},
    },
    error::ApiError,
    types::DeviceId,
    validation::{
        generate_device_token, validate_attestation_format, validate_capabilities, validate_csr,
        validate_firmware_version, validate_x25519_public_key,
    },
    vault::{create_vault_client, issue_certificate},
    AppState,
};

/// Device registration request
#[derive(Debug, Deserialize)]
pub struct RegisterDeviceRequest {
    pub device_id: String,
    pub public_key: String,
    pub firmware_version: String,
    pub capabilities: Vec<String>,
    pub attestation: Option<AttestationData>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
pub struct AttestationData {
    pub format: String,
    pub evidence: String,
    pub nonce: String,
}

/// Device registration response
#[derive(Debug, Serialize)]
pub struct RegisterDeviceResponse {
    pub device_token: String,
    pub pairing_code: String,
    pub registered_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Device pairing request
#[derive(Debug, Deserialize)]
pub struct PairDeviceRequest {
    pub pairing_code: String,
    pub device_cert_csr: String,
    pub telemetry_topics: Vec<String>,
    pub policy_version: Option<String>,
}

/// Device pairing response
#[derive(Debug, Serialize)]
pub struct PairDeviceResponse {
    pub device_certificate: String,
    pub policy_bundle: PolicyBundle,
    pub session_endpoint: String,
}

#[derive(Debug, Serialize)]
pub struct PolicyBundle {
    pub version: String,
    pub sha512: String,
    pub signed_payload: String,
}

/// Creates device management routes
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(register_device))
        .route("/:device_id/pair", post(pair_device))
}

/// POST /devices - Register a new device
#[tracing::instrument(skip(state, req), fields(device_id = %req.device_id))]
async fn register_device(
    State(state): State<AppState>,
    Json(req): Json<RegisterDeviceRequest>,
) -> Result<impl IntoResponse, ApiError> {
    tracing::info!("Registering device: {}", req.device_id);

    // Extract trace_id for audit logging
    let trace_id = crate::error::extract_trace_id();

    // Validate device_id
    let device_id = DeviceId(req.device_id.clone());
    device_id.validate()?;

    // Validate public key (X25519, 32 bytes)
    let public_key = validate_x25519_public_key(&req.public_key)?;

    // Validate firmware version (SemVer)
    let _firmware_version = validate_firmware_version(&req.firmware_version)?;

    // Validate capabilities
    validate_capabilities(&req.capabilities)?;

    // Validate attestation if present
    let (attestation_format, attestation_evidence, attestation_nonce) =
        if let Some(ref attestation) = req.attestation {
            validate_attestation_format(&attestation.format)?;

            let evidence = base64::engine::general_purpose::STANDARD
                .decode(&attestation.evidence)
                .map_err(|e| ApiError::Validation(format!("Invalid attestation evidence base64: {}", e)))?;

            (
                Some(attestation.format.clone()),
                Some(evidence),
                Some(attestation.nonce.clone()),
            )
        } else {
            (None, None, None)
        };

    // Generate secure device token
    let device_token = generate_device_token();

    // Create device record
    let params = CreateDeviceParams {
        device_id: device_id.clone(),
        public_key,
        firmware_version: req.firmware_version,
        capabilities: req.capabilities,
        attestation_format,
        attestation_evidence,
        attestation_nonce,
        metadata: req.metadata,
        device_token: device_token.clone(),
    };

    let device = create_device(&state.db_pool, params).await?;

    // Generate pairing code (10-minute TTL)
    let (pairing_code, expires_at) = create_pairing_code(&state.db_pool, &device_id).await?;

    // Record audit event
    record_audit_event(
        &state.db_pool,
        CreateAuditEventParams {
            category: AuditCategory::DeviceRegistration,
            actor: "system".to_string(),
            device_id: Some(device_id.0.clone()),
            outcome: AuditOutcome::Success,
            details: Some(serde_json::json!({
                "firmware_version": device.firmware_version,
                "capabilities": device.capabilities,
            })),
            trace_id: Some(trace_id),
        },
    )
    .await?;

    tracing::info!("Device registered successfully: {}", device_id.0);

    Ok((
        StatusCode::CREATED,
        Json(RegisterDeviceResponse {
            device_token,
            pairing_code,
            registered_at: device.registered_at,
            expires_at,
        }),
    ))
}

/// POST /devices/{device_id}/pair - Pair a device and issue certificate
#[tracing::instrument(skip(state, req), fields(device_id = %device_id))]
async fn pair_device(
    State(state): State<AppState>,
    Path(device_id): Path<String>,
    Json(req): Json<PairDeviceRequest>,
) -> Result<impl IntoResponse, ApiError> {
    tracing::info!("Pairing device: {}", device_id);

    // Extract trace_id for audit logging
    let trace_id = crate::error::extract_trace_id();

    // Validate device_id
    let device_id = DeviceId(device_id);
    device_id.validate()?;

    // Get device record
    let device = get_device(&state.db_pool, &device_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Device not found: {}", device_id.0)))?;

    // Check device status
    if device.status != "pending" {
        return Err(ApiError::State(format!(
            "Device is not in pending state: {}",
            device.status
        )));
    }

    // Validate and consume pairing code
    validate_and_consume_pairing_code(&state.db_pool, &device_id, &req.pairing_code).await?;

    // Validate CSR
    let csr_pem_bytes = validate_csr(&req.device_cert_csr)?;
    let csr_pem = String::from_utf8(csr_pem_bytes)
        .map_err(|e| ApiError::Validation(format!("CSR is not valid UTF-8: {}", e)))?;

    // Issue certificate via Vault PKI
    let vault_client = create_vault_client(&state.vault_config)?;
    let certificate = issue_certificate(
        &vault_client,
        &state.vault_config,
        &device_id.0, // Common name = device_id
        &csr_pem,
    )
    .await?;

    // Mark device as paired
    mark_device_paired(&state.db_pool, &device_id, &certificate.serial_number).await?;

    // TODO: Get policy bundle from policy engine (Task 2.2 integration)
    // For now, return a placeholder
    let policy_version = req.policy_version.unwrap_or_else(|| "2025.01.01".to_string());
    let policy_payload = serde_json::json!({
        "version": policy_version,
        "qos": {
            "telemetry": { "priority": 3, "latency_budget_ms": 150 },
            "control": { "priority": 1, "latency_budget_ms": 30 }
        },
        "encryption": {
            "ciphers": ["chacha20-poly1305"],
            "fallback": null
        },
        "features": {
            "ota_update": false,
            "diagnostics": true
        }
    });

    // TODO: Sign policy bundle with Ed25519 (Task 2.4 crypto integration)
    // For now, use placeholder signature
    let policy_payload_str = serde_json::to_string(&policy_payload)
        .map_err(|e| ApiError::Internal(format!("Failed to serialize policy: {}", e)))?;

    let policy_sha512 = format!("{:x}", sha2::Sha512::digest(policy_payload_str.as_bytes()));
    let policy_signed = base64::engine::general_purpose::STANDARD.encode(&policy_payload_str);

    // TODO: Get session endpoint from configuration
    let session_endpoint = state
        .config
        .session_endpoint
        .clone()
        .unwrap_or_else(|| "quic://127.0.0.1:7843".to_string());

    // Encode certificate as base64
    let device_certificate_b64 = base64::engine::general_purpose::STANDARD.encode(&certificate.certificate);

    // Record audit event
    record_audit_event(
        &state.db_pool,
        CreateAuditEventParams {
            category: AuditCategory::DevicePairing,
            actor: "system".to_string(),
            device_id: Some(device_id.0.clone()),
            outcome: AuditOutcome::Success,
            details: Some(serde_json::json!({
                "certificate_serial": certificate.serial_number,
                "policy_version": policy_version,
            })),
            trace_id: Some(trace_id),
        },
    )
    .await?;

    // TODO: Fire key rotation notification event (Task 2.4 integration)
    // This would notify the key rotation scheduler about the new device

    tracing::info!("Device paired successfully: {}", device_id.0);

    Ok((
        StatusCode::OK,
        Json(PairDeviceResponse {
            device_certificate: device_certificate_b64,
            policy_bundle: PolicyBundle {
                version: policy_version,
                sha512: policy_sha512,
                signed_payload: policy_signed,
            },
            session_endpoint,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_id_validation() {
        let valid_id = DeviceId("HL-EDGE-0001".to_string());
        assert!(valid_id.validate().is_ok());

        let invalid_id = DeviceId("invalid!".to_string());
        assert!(invalid_id.validate().is_err());
    }
}
