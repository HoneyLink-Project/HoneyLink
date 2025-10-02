// Session management API endpoints
//
// Implements POST /sessions for multi-stream session creation with QoS allocation,
// key derivation, and FEC parameter calculation.

use crate::db::{self, audit, devices, sessions};
use crate::error::ApiError;
use crate::{types::SessionId, AppState};
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::Utc;
use honeylink_crypto::key_derivation::{DeriveContext, KeyDerivation};
use honeylink_qos_scheduler::scheduler::{QoSPriority, QoSScheduler, StreamMode, StreamRequest};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

/// FEC (Forward Error Correction) parameters for a stream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FecParams {
    /// Number of data shards
    pub data_shards: usize,
    /// Number of parity shards (redundancy)
    pub parity_shards: usize,
}

impl FecParams {
    /// Calculates FEC parameters based on QoS priority
    ///
    /// FEC redundancy strategy:
    /// - Burst (high-priority video): 50% parity (10 data + 5 parity)
    /// - Normal (telemetry): 20% parity (10 data + 2 parity)
    /// - Latency (control): 10% parity (10 data + 1 parity)
    ///
    /// Higher redundancy provides better error correction but increases overhead.
    /// Latency-sensitive streams use minimal redundancy for speed.
    pub fn from_priority(priority: QoSPriority) -> Self {
        const DATA_SHARDS: usize = 10;

        let parity_shards = match priority {
            QoSPriority::Burst => 5,    // 50% redundancy
            QoSPriority::Normal => 2,   // 20% redundancy
            QoSPriority::Latency => 1,  // 10% redundancy
        };

        Self {
            data_shards: DATA_SHARDS,
            parity_shards,
        }
    }
}

/// QoS configuration for a stream
#[derive(Debug, Clone, Deserialize)]
pub struct QoSConfig {
    /// Priority level (burst, normal, latency)
    pub priority: String,
    /// Requested bandwidth in kbps
    pub bandwidth_kbps: u32,
}

/// Stream request in session creation
#[derive(Debug, Clone, Deserialize)]
pub struct StreamSpec {
    /// Stream name (e.g., "telemetry", "video")
    pub name: String,
    /// Stream mode (reliable or unreliable)
    pub mode: String,
    /// QoS configuration
    pub qos: QoSConfig,
}

/// Request body for POST /sessions
#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    /// Device identifier (must be paired)
    pub device_id: String,
    /// List of streams to create
    pub streams: Vec<StreamSpec>,
    /// Session TTL in seconds (default: 3600 = 1 hour)
    #[serde(default = "default_ttl")]
    pub ttl_seconds: i64,
}

fn default_ttl() -> i64 {
    3600 // 1 hour
}

/// Stream information in session response
#[derive(Debug, Clone, Serialize)]
pub struct StreamInfo {
    /// Stream UUID
    pub stream_id: Uuid,
    /// Stream name
    pub name: String,
    /// Connection identifier for transport layer
    pub connection_id: String,
    /// Base64-encoded stream key material (32 bytes)
    pub key_material: String,
    /// FEC parameters
    pub fec: FecParams,
}

/// Response body for POST /sessions
#[derive(Debug, Serialize)]
pub struct CreateSessionResponse {
    /// Session identifier (UUIDv7)
    pub session_id: Uuid,
    /// Session expiration timestamp
    pub expires_at: String,
    /// Session endpoint URL (e.g., "quic://127.0.0.1:7843")
    pub session_endpoint: String,
    /// List of allocated streams
    pub streams: Vec<StreamInfo>,
}

/// Creates a new session router
pub fn routes() -> Router<AppState> {
    Router::new().route("/", post(create_session))
}

/// POST /sessions - Create a new multi-stream session
///
/// # Request
/// ```json
/// {
///   "device_id": "device-12345",
///   "streams": [
///     {
///       "name": "telemetry",
///       "mode": "reliable",
///       "qos": { "priority": "normal", "bandwidth_kbps": 100 }
///     },
///     {
///       "name": "video",
///       "mode": "unreliable",
///       "qos": { "priority": "burst", "bandwidth_kbps": 5000 }
///     }
///   ],
///   "ttl_seconds": 3600
/// }
/// ```
///
/// # Response (201 Created)
/// ```json
/// {
///   "session_id": "01JAY...",
///   "expires_at": "2025-10-02T11:00:00Z",
///   "session_endpoint": "quic://127.0.0.1:7843",
///   "streams": [
///     {
///       "stream_id": "01JAY...",
///       "name": "telemetry",
///       "connection_id": "conn-001",
///       "key_material": "base64-encoded-key",
///       "fec": { "data_shards": 10, "parity_shards": 2 }
///     }
///   ]
/// }
/// ```
///
/// # Errors
/// - 400 Validation: Invalid input (empty streams, invalid QoS priority, invalid mode)
/// - 404 NotFound: Device not found
/// - 409 State: Device not paired (must be paired first)
/// - 503 Dependency: QoS allocation failure, database error
async fn create_session(
    State(state): State<AppState>,
    Json(req): Json<CreateSessionRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Get OpenTelemetry trace ID for correlation
    let trace_id = opentelemetry::trace::TraceContextExt::span(
        &tracing::Span::current().context(),
    )
    .span_context()
    .trace_id()
    .to_string();

    // Step 1: Validate device exists and is paired
    let device = devices::get_device(&state.db_pool, &req.device_id)
        .await
        .map_err(|e| match e {
            ApiError::NotFound(_) => ApiError::NotFound(format!("Device '{}' not found", req.device_id)),
            _ => e,
        })?;

    if device.status != devices::DeviceStatus::Paired {
        return Err(ApiError::State(format!(
            "Device must be paired before creating sessions (current status: {})",
            device.status
        )));
    }

    // Step 2: Validate stream specifications
    if req.streams.is_empty() {
        return Err(ApiError::Validation(
            "At least one stream is required".to_string(),
        ));
    }

    // Step 3: Parse and validate stream requests for QoS Scheduler
    let mut stream_requests = Vec::with_capacity(req.streams.len());

    for stream_spec in &req.streams {
        // Parse priority
        let priority = match stream_spec.qos.priority.to_lowercase().as_str() {
            "burst" => QoSPriority::Burst,
            "normal" => QoSPriority::Normal,
            "latency" => QoSPriority::Latency,
            _ => {
                return Err(ApiError::Validation(format!(
                    "Invalid QoS priority '{}' for stream '{}' (valid: burst, normal, latency)",
                    stream_spec.qos.priority, stream_spec.name
                )));
            }
        };

        // Parse mode
        let mode = match stream_spec.mode.to_lowercase().as_str() {
            "reliable" => StreamMode::Reliable,
            "unreliable" => StreamMode::Unreliable,
            _ => {
                return Err(ApiError::Validation(format!(
                    "Invalid stream mode '{}' for stream '{}' (valid: reliable, unreliable)",
                    stream_spec.mode, stream_spec.name
                )));
            }
        };

        stream_requests.push(StreamRequest {
            name: stream_spec.name.clone(),
            mode,
            priority,
            bandwidth_kbps: stream_spec.qos.bandwidth_kbps,
        });
    }

    // Step 4: Allocate streams via QoS Scheduler
    let mut qos_scheduler = QoSScheduler::new(); // TODO: Use shared scheduler instance from AppState
    let allocations = qos_scheduler
        .allocate_streams(&stream_requests)
        .map_err(|e| ApiError::Dependency(format!("QoS allocation failed: {}", e)))?;

    // Step 5: Generate session ID (UUIDv7)
    let session_id = SessionId::new().0;

    // Step 6: Derive session key material via HKDF
    // Use device public key as input keying material (IKM)
    // Device public key is 32 bytes (X25519)
    let device_public_key = URL_SAFE_NO_PAD.decode(device.public_key.as_bytes())
        .map_err(|e| ApiError::Internal(format!("Failed to decode device public key: {}", e)))?;

    if device_public_key.len() != 32 {
        return Err(ApiError::Internal(
            "Device public key must be 32 bytes".to_string(),
        ));
    }

    // Derive session key (32 bytes)
    let session_context = DeriveContext::session(&req.device_id, &session_id.to_string());
    let session_key = KeyDerivation::derive_with_context(&device_public_key, &session_context, 32)
        .map_err(|e| ApiError::Internal(format!("Failed to derive session key: {}", e)))?;

    // Step 7: Derive per-stream keys and calculate FEC parameters
    let mut stream_infos = Vec::with_capacity(allocations.len());

    for allocation in &allocations {
        // Derive stream key from session key
        let stream_context = DeriveContext::stream(
            &session_id.to_string(),
            &allocation.stream_id.to_string(),
        );
        let stream_key = KeyDerivation::derive_with_context(&session_key, &stream_context, 32)
            .map_err(|e| ApiError::Internal(format!("Failed to derive stream key: {}", e)))?;

        // Encode key as base64url
        let key_material = URL_SAFE_NO_PAD.encode(&stream_key);

        // Calculate FEC parameters based on QoS priority
        let fec = FecParams::from_priority(allocation.priority);

        stream_infos.push(StreamInfo {
            stream_id: allocation.stream_id.0,
            name: allocation.name.clone(),
            connection_id: allocation.connection_id.clone(),
            key_material,
            fec,
        });
    }

    // Step 8: Store session in database
    // Convert stream allocations to JSONB for storage
    let streams_json = json!(stream_infos
        .iter()
        .map(|s| {
            json!({
                "stream_id": s.stream_id,
                "name": s.name,
                "connection_id": s.connection_id,
                "fec": s.fec,
            })
        })
        .collect::<Vec<_>>());

    let session_params = sessions::CreateSessionParams {
        session_id,
        device_id: req.device_id.clone(),
        streams: streams_json,
        key_material: session_key.to_vec(), // Store session key for future derivations
        ttl_seconds: req.ttl_seconds,
        endpoint: state
            .config
            .session_endpoint
            .clone()
            .unwrap_or_else(|| "quic://127.0.0.1:7843".to_string()),
    };

    let session = sessions::create_session(&state.db_pool, session_params)
        .await
        .map_err(|e| match e {
            ApiError::Conflict(_) => {
                ApiError::Internal("Session ID collision (UUIDv7 conflict)".to_string())
            }
            _ => e,
        })?;

    // Step 9: Record audit event
    let audit_params = audit::CreateAuditEventParams {
        category: audit::AuditCategory::SessionCreation,
        actor: format!("device:{}", req.device_id),
        device_id: Some(req.device_id.clone()),
        outcome: audit::AuditOutcome::Success,
        details: json!({
            "session_id": session_id.to_string(),
            "stream_count": stream_infos.len(),
            "total_bandwidth_kbps": stream_requests.iter().map(|s| s.bandwidth_kbps).sum::<u32>(),
            "ttl_seconds": req.ttl_seconds,
        }),
        trace_id: Some(trace_id),
    };

    let _ = audit::record_audit_event(&state.db_pool, audit_params).await; // Best effort

    // Step 10: Return response
    let response = CreateSessionResponse {
        session_id,
        expires_at: session.expires_at.to_rfc3339(),
        session_endpoint: session.endpoint.clone(),
        streams: stream_infos,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fec_params_from_priority() {
        let burst = FecParams::from_priority(QoSPriority::Burst);
        assert_eq!(burst.data_shards, 10);
        assert_eq!(burst.parity_shards, 5); // 50%

        let normal = FecParams::from_priority(QoSPriority::Normal);
        assert_eq!(normal.data_shards, 10);
        assert_eq!(normal.parity_shards, 2); // 20%

        let latency = FecParams::from_priority(QoSPriority::Latency);
        assert_eq!(latency.data_shards, 10);
        assert_eq!(latency.parity_shards, 1); // 10%
    }

    #[test]
    fn test_default_ttl() {
        assert_eq!(default_ttl(), 3600);
    }
}
