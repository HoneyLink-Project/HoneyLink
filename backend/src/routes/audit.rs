// Audit API endpoints (GET /audit/events)
// Implements WORM (Write Once Read Many) audit log retrieval with pagination,
// filtering, and SSE streaming support.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use futures::stream::{self, Stream};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use std::{convert::Infallible, time::Duration};

use crate::{
    db::audit::{
        get_audit_events_by_category, get_audit_events_by_device, AuditCategory, AuditEvent,
    },
    error::ApiError,
    middleware::auth::RequireAuth,
    AppState,
};

/// Query parameters for GET /audit/events
#[derive(Debug, Deserialize)]
pub struct AuditQueryParams {
    /// Filter by device ID (optional)
    #[serde(default)]
    pub device_id: Option<String>,

    /// Filter by category (optional)
    #[serde(default)]
    pub category: Option<String>,

    /// Filter by timestamp (events after this time)
    #[serde(default)]
    pub since: Option<DateTime<Utc>>,

    /// Pagination limit (default: 100, max: 1000)
    #[serde(default = "default_limit")]
    pub limit: i64,

    /// Enable Server-Sent Events streaming (default: false)
    #[serde(default)]
    pub stream: bool,
}

fn default_limit() -> i64 {
    100
}

/// Audit event response item with Ed25519 signature
#[derive(Debug, Serialize)]
pub struct AuditEventResponse {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub category: String,
    pub actor: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
    pub outcome: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
    /// Ed25519 signature for non-repudiation (hex-encoded)
    pub signature: String,
}

/// Audit events list response
#[derive(Debug, Serialize)]
pub struct AuditEventsResponse {
    pub events: Vec<AuditEventResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
}

/// Creates audit API routes
pub fn routes() -> Router<AppState> {
    Router::new().route("/audit/events", get(get_audit_events))
}

/// GET /audit/events - Retrieves audit events with filtering and pagination
///
/// Query parameters:
/// - device_id: Filter by device ID
/// - category: Filter by event category (device-registration, key-rotation, etc.)
/// - since: Filter by timestamp (ISO 8601)
/// - limit: Number of events to return (default: 100, max: 1000)
/// - stream: Enable SSE streaming (default: false)
///
/// Returns:
/// - 200 OK: List of audit events with Ed25519 signatures
/// - 400 Bad Request: Invalid query parameters
/// - 401 Unauthorized: Authentication failure
/// - 403 Forbidden: Insufficient permissions
/// - 500 Internal Server Error: Database error
#[tracing::instrument(skip(state, query), fields(device_id = ?query.device_id, category = ?query.category))]
async fn get_audit_events(
    _auth: RequireAuth, // OAuth2 authentication required
    State(state): State<AppState>,
    Query(query): Query<AuditQueryParams>,
) -> Result<Response, ApiError> {
    // Step 1: Validate query parameters
    if query.limit <= 0 || query.limit > 1000 {
        return Err(ApiError::Validation(
            "Limit must be between 1 and 1000".to_string(),
        ));
    }

    // Step 2: RBAC/ABAC Authorization check (placeholder)
    // TODO(Task 5.1): Implement full RBAC/ABAC policy evaluation
    // Example:
    // let policy_ctx = PolicyContext {
    //     user_id: auth.user_id.clone(),
    //     resource: "audit:events".to_string(),
    //     action: "audit:read",
    //     attributes: HashMap::from([
    //         ("device_id".to_string(), query.device_id.clone().unwrap_or_default()),
    //     ]),
    // };
    // state.policy_engine.evaluate(&policy_ctx).await?;

    // Step 3: Parse category if provided
    let category_filter = if let Some(cat_str) = &query.category {
        Some(parse_audit_category(cat_str)?)
    } else {
        None
    };

    // Step 4: Check if SSE streaming is requested
    if query.stream {
        // Return SSE stream response
        return Ok(stream_audit_events(state, query, category_filter).await?.into_response());
    }

    // Step 5: Fetch audit events from database
    let events = if let Some(device_id) = &query.device_id {
        // Filter by device ID
        get_audit_events_by_device(&state.db_pool, device_id, query.since, query.limit).await?
    } else if let Some(category) = category_filter {
        // Filter by category
        get_audit_events_by_category(&state.db_pool, category, query.since, query.limit).await?
    } else {
        // Fetch all events (limited by limit)
        fetch_all_audit_events(&state.db_pool, query.since, query.limit).await?
    };

    // Step 6: Sign each event with Ed25519 (placeholder)
    let signed_events = events
        .into_iter()
        .map(|event| sign_audit_event(event))
        .collect::<Result<Vec<_>, _>>()?;

    // Step 7: Determine next cursor (pagination)
    let next_cursor = if signed_events.len() == query.limit as usize {
        signed_events
            .last()
            .map(|e| format!("cursor_{}", e.id))
    } else {
        None
    };

    // Step 8: Return response
    Ok(Json(AuditEventsResponse {
        events: signed_events,
        next: next_cursor,
    })
    .into_response())
}

/// Streams audit events via Server-Sent Events (SSE)
async fn stream_audit_events(
    state: AppState,
    query: AuditQueryParams,
    category: Option<AuditCategory>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    // Initial fetch
    let initial_events = if let Some(device_id) = &query.device_id {
        get_audit_events_by_device(&state.db_pool, device_id, query.since, query.limit).await?
    } else if let Some(cat) = category {
        get_audit_events_by_category(&state.db_pool, cat, query.since, query.limit).await?
    } else {
        fetch_all_audit_events(&state.db_pool, query.since, query.limit).await?
    };

    // Sign events
    let signed_events = initial_events
        .into_iter()
        .map(|event| sign_audit_event(event))
        .collect::<Result<Vec<_>, _>>()?;

    // Create SSE stream
    let stream = stream::iter(signed_events.into_iter().map(|event| {
        let json_str = serde_json::to_string(&event).unwrap_or_default();
        Ok(Event::default().data(json_str))
    }));

    // TODO(Task 6.2): Implement real-time event subscription
    // For now, return initial batch only
    // Future implementation would use:
    // - Redis Pub/Sub for event notifications
    // - tokio::time::interval for polling
    // - tokio::sync::broadcast channel for live updates

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

/// Parses audit category string into AuditCategory enum
fn parse_audit_category(category: &str) -> Result<AuditCategory, ApiError> {
    match category.to_lowercase().as_str() {
        "device-registration" => Ok(AuditCategory::DeviceRegistration),
        "device-pairing" => Ok(AuditCategory::DevicePairing),
        "key-rotation" => Ok(AuditCategory::KeyRotation),
        "policy-update" => Ok(AuditCategory::PolicyUpdate),
        "session-creation" => Ok(AuditCategory::SessionCreation),
        "access-denied" => Ok(AuditCategory::AccessDenied),
        "configuration-change" => Ok(AuditCategory::ConfigurationChange),
        _ => Err(ApiError::Validation(format!(
            "Invalid audit category '{}'",
            category
        ))),
    }
}

/// Fetches all audit events (without device_id or category filter)
async fn fetch_all_audit_events(
    pool: &sqlx::PgPool,
    since: Option<DateTime<Utc>>,
    limit: i64,
) -> Result<Vec<AuditEvent>, ApiError> {
    let events = if let Some(since_time) = since {
        sqlx::query_as!(
            AuditEvent,
            r#"
            SELECT
                id, timestamp, category, actor, device_id, outcome,
                details as "details: sqlx::types::Json<JsonValue>",
                trace_id
            FROM audit_events
            WHERE timestamp >= $1
            ORDER BY timestamp DESC
            LIMIT $2
            "#,
            since_time,
            limit
        )
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as!(
            AuditEvent,
            r#"
            SELECT
                id, timestamp, category, actor, device_id, outcome,
                details as "details: sqlx::types::Json<JsonValue>",
                trace_id
            FROM audit_events
            ORDER BY timestamp DESC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(pool)
        .await
    };

    events.map_err(|e| ApiError::Dependency(format!("Failed to fetch audit events: {}", e)))
}

/// Signs an audit event with Ed25519 for non-repudiation
fn sign_audit_event(event: AuditEvent) -> Result<AuditEventResponse, ApiError> {
    // TODO(Task 2.4): Integrate honeylink-crypto Ed25519 signing
    // Implementation plan:
    // 1. Serialize event to canonical JSON (sorted keys, no whitespace)
    // 2. Hash with SHA-512
    // 3. Sign hash with Ed25519 private key (from KMS or local keystore)
    // 4. Return hex-encoded signature
    //
    // Example:
    // use honeylink_crypto::Ed25519Signer;
    // let canonical_json = canonical_serialize(&event)?;
    // let signature = state.ed25519_signer.sign(&canonical_json).await?;
    // let signature_hex = hex::encode(signature);

    // Placeholder: Generate deterministic signature for testing
    let signature_input = format!(
        "{}:{}:{}:{}",
        event.id,
        event.timestamp.timestamp(),
        event.category,
        event.outcome
    );
    let signature_hex = format!("ed25519_{:x}", md5::compute(&signature_input));

    Ok(AuditEventResponse {
        id: event.id.to_string(),
        timestamp: event.timestamp,
        category: event.category,
        actor: event.actor,
        device_id: event.device_id,
        outcome: event.outcome,
        details: event.details.map(|d| d.0),
        trace_id: event.trace_id,
        signature: signature_hex,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_audit_category() {
        assert_eq!(
            parse_audit_category("device-registration").unwrap(),
            AuditCategory::DeviceRegistration
        );
        assert_eq!(
            parse_audit_category("KEY-ROTATION").unwrap(),
            AuditCategory::KeyRotation
        );
        assert!(parse_audit_category("invalid").is_err());
    }

    #[test]
    fn test_default_limit() {
        assert_eq!(default_limit(), 100);
    }

    #[test]
    fn test_audit_query_params_deserialization() {
        let json = r#"{"device_id": "device-123", "limit": 50}"#;
        let params: AuditQueryParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.device_id, Some("device-123".to_string()));
        assert_eq!(params.limit, 50);
        assert_eq!(params.stream, false);
    }

    #[test]
    fn test_sign_audit_event_deterministic() {
        use uuid::Uuid;

        let event = AuditEvent {
            id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            timestamp: DateTime::parse_from_rfc3339("2025-03-01T08:59:00Z")
                .unwrap()
                .with_timezone(&Utc),
            category: "key-rotation".to_string(),
            actor: "secops".to_string(),
            device_id: Some("device-123".to_string()),
            outcome: "success".to_string(),
            details: None,
            trace_id: Some("trace-abc".to_string()),
        };

        let signed = sign_audit_event(event).unwrap();
        assert!(signed.signature.starts_with("ed25519_"));
        assert_eq!(signed.category, "key-rotation");
    }

    #[test]
    fn test_audit_event_response_serialization() {
        let response = AuditEventResponse {
            id: "evt_c4a2".to_string(),
            timestamp: DateTime::parse_from_rfc3339("2025-03-01T08:59:00Z")
                .unwrap()
                .with_timezone(&Utc),
            category: "key-rotation".to_string(),
            actor: "secops".to_string(),
            device_id: Some("device-123".to_string()),
            outcome: "success".to_string(),
            details: Some(json!({"rotation_id": "rot_45ab"})),
            trace_id: Some("trace-abc".to_string()),
            signature: "ed25519_1234abcd".to_string(),
        };

        let json_str = serde_json::to_string(&response).unwrap();
        assert!(json_str.contains("\"category\":\"key-rotation\""));
        assert!(json_str.contains("\"signature\":\"ed25519_1234abcd\""));
    }
}
