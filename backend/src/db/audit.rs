// Audit log database operations (WORM compliance)

use crate::error::ApiError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use uuid::Uuid;

/// Audit event category
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AuditCategory {
    DeviceRegistration,
    DevicePairing,
    KeyRotation,
    PolicyUpdate,
    SessionCreation,
    AccessDenied,
    ConfigurationChange,
}

impl std::fmt::Display for AuditCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditCategory::DeviceRegistration => write!(f, "device-registration"),
            AuditCategory::DevicePairing => write!(f, "device-pairing"),
            AuditCategory::KeyRotation => write!(f, "key-rotation"),
            AuditCategory::PolicyUpdate => write!(f, "policy-update"),
            AuditCategory::SessionCreation => write!(f, "session-creation"),
            AuditCategory::AccessDenied => write!(f, "access-denied"),
            AuditCategory::ConfigurationChange => write!(f, "configuration-change"),
        }
    }
}

/// Audit event outcome
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AuditOutcome {
    Success,
    Failure,
}

impl std::fmt::Display for AuditOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditOutcome::Success => write!(f, "success"),
            AuditOutcome::Failure => write!(f, "failure"),
        }
    }
}

/// Audit event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub category: String,
    pub actor: String,
    pub device_id: Option<String>,
    pub outcome: String,
    pub details: Option<sqlx::types::Json<JsonValue>>,
    pub trace_id: Option<String>,
}

/// Parameters for creating an audit event
#[derive(Debug, Clone)]
pub struct CreateAuditEventParams {
    pub category: AuditCategory,
    pub actor: String,
    pub device_id: Option<String>,
    pub outcome: AuditOutcome,
    pub details: Option<JsonValue>,
    pub trace_id: Option<String>,
}

/// Records an audit event (append-only)
/// Returns the event ID
pub async fn record_audit_event(
    pool: &PgPool,
    params: CreateAuditEventParams,
) -> Result<Uuid, ApiError> {
    let details_json = params.details.map(sqlx::types::Json);

    let record = sqlx::query!(
        r#"
        INSERT INTO audit_events (category, actor, device_id, outcome, details, trace_id)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
        params.category.to_string(),
        params.actor,
        params.device_id,
        params.outcome.to_string(),
        details_json as Option<sqlx::types::Json<JsonValue>>,
        params.trace_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| ApiError::Dependency(format!("Failed to record audit event: {}", e)))?;

    Ok(record.id)
}

/// Gets audit events by device ID (read-only)
pub async fn get_audit_events_by_device(
    pool: &PgPool,
    device_id: &str,
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
            WHERE device_id = $1 AND timestamp >= $2
            ORDER BY timestamp DESC
            LIMIT $3
            "#,
            device_id,
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
            WHERE device_id = $1
            ORDER BY timestamp DESC
            LIMIT $2
            "#,
            device_id,
            limit
        )
        .fetch_all(pool)
        .await
    };

    events.map_err(|e| ApiError::Dependency(format!("Failed to fetch audit events: {}", e)))
}

/// Gets audit events by category (read-only)
pub async fn get_audit_events_by_category(
    pool: &PgPool,
    category: AuditCategory,
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
            WHERE category = $1 AND timestamp >= $2
            ORDER BY timestamp DESC
            LIMIT $3
            "#,
            category.to_string(),
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
            WHERE category = $1
            ORDER BY timestamp DESC
            LIMIT $2
            "#,
            category.to_string(),
            limit
        )
        .fetch_all(pool)
        .await
    };

    events.map_err(|e| ApiError::Dependency(format!("Failed to fetch audit events: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_category_display() {
        assert_eq!(AuditCategory::DeviceRegistration.to_string(), "device-registration");
        assert_eq!(AuditCategory::KeyRotation.to_string(), "key-rotation");
    }

    #[test]
    fn test_audit_outcome_display() {
        assert_eq!(AuditOutcome::Success.to_string(), "success");
        assert_eq!(AuditOutcome::Failure.to_string(), "failure");
    }
}
