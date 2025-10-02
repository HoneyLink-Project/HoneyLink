// Session database operations
//
// Provides CRUD operations for session management with multi-stream configuration.
// Sessions have TTL-based expiration and are automatically cleaned up.

use crate::error::ApiError;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use uuid::Uuid;

/// Session status enum matching database type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "session_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    Active,
    Expired,
    Terminated,
}

impl std::fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionStatus::Active => write!(f, "active"),
            SessionStatus::Expired => write!(f, "expired"),
            SessionStatus::Terminated => write!(f, "terminated"),
        }
    }
}

/// Session record from database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_id: Uuid,
    pub device_id: String,
    pub streams: JsonValue, // JSONB array of stream configurations
    #[serde(skip_serializing)] // Don't expose key material in logs
    pub key_material: Vec<u8>,
    pub expires_at: DateTime<Utc>,
    pub status: SessionStatus,
    pub endpoint: String,
    pub created_at: DateTime<Utc>,
    pub terminated_at: Option<DateTime<Utc>>,
}

/// Parameters for creating a new session
pub struct CreateSessionParams {
    pub session_id: Uuid,
    pub device_id: String,
    pub streams: JsonValue,
    pub key_material: Vec<u8>,
    pub ttl_seconds: i64,
    pub endpoint: String,
}

/// Creates a new session in the database
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `params` - Session creation parameters
///
/// # Returns
/// * `Ok(Session)` - Created session record
/// * `Err(ApiError::Conflict)` - Session ID already exists
/// * `Err(ApiError::Dependency)` - Database error or device not found
///
/// # Example
/// ```no_run
/// use honeylink_control_plane::db::sessions::{create_session, CreateSessionParams};
/// use uuid::Uuid;
///
/// # async fn example(pool: sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
/// let params = CreateSessionParams {
///     session_id: Uuid::now_v7(),
///     device_id: "device-12345".to_string(),
///     streams: serde_json::json!([]),
///     key_material: vec![0u8; 32],
///     ttl_seconds: 3600,
///     endpoint: "quic://127.0.0.1:7843".to_string(),
/// };
///
/// let session = create_session(&pool, params).await?;
/// # Ok(())
/// # }
/// ```
pub async fn create_session(pool: &PgPool, params: CreateSessionParams) -> Result<Session, ApiError> {
    let expires_at = Utc::now() + Duration::seconds(params.ttl_seconds);

    let session = sqlx::query_as!(
        Session,
        r#"
        INSERT INTO sessions (session_id, device_id, streams, key_material, expires_at, status, endpoint)
        VALUES ($1, $2, $3, $4, $5, 'active', $6)
        RETURNING
            session_id,
            device_id,
            streams,
            key_material,
            expires_at,
            status AS "status: SessionStatus",
            endpoint,
            created_at,
            terminated_at
        "#,
        params.session_id,
        params.device_id,
        params.streams,
        params.key_material,
        expires_at,
        params.endpoint,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(db_err) if db_err.constraint().is_some() => {
            if db_err.constraint().unwrap().contains("pkey") {
                ApiError::Conflict("Session ID already exists".to_string())
            } else if db_err.constraint().unwrap().contains("fk_device") {
                ApiError::NotFound("Device not found".to_string())
            } else {
                ApiError::Dependency(format!("Database constraint violation: {}", db_err))
            }
        }
        _ => ApiError::Dependency(format!("Failed to create session: {}", e)),
    })?;

    Ok(session)
}

/// Retrieves a session by ID
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `session_id` - Session UUID to retrieve
///
/// # Returns
/// * `Ok(Session)` - Session record
/// * `Err(ApiError::NotFound)` - Session not found
/// * `Err(ApiError::Dependency)` - Database error
pub async fn get_session(pool: &PgPool, session_id: Uuid) -> Result<Session, ApiError> {
    let session = sqlx::query_as!(
        Session,
        r#"
        SELECT
            session_id,
            device_id,
            streams,
            key_material,
            expires_at,
            status AS "status: SessionStatus",
            endpoint,
            created_at,
            terminated_at
        FROM sessions
        WHERE session_id = $1
        "#,
        session_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| ApiError::Dependency(format!("Failed to fetch session: {}", e)))?
    .ok_or_else(|| ApiError::NotFound("Session not found".to_string()))?;

    Ok(session)
}

/// Retrieves all active sessions for a device
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `device_id` - Device identifier
///
/// # Returns
/// * `Ok(Vec<Session>)` - List of active sessions
/// * `Err(ApiError::Dependency)` - Database error
pub async fn get_device_sessions(pool: &PgPool, device_id: &str) -> Result<Vec<Session>, ApiError> {
    let sessions = sqlx::query_as!(
        Session,
        r#"
        SELECT
            session_id,
            device_id,
            streams,
            key_material,
            expires_at,
            status AS "status: SessionStatus",
            endpoint,
            created_at,
            terminated_at
        FROM sessions
        WHERE device_id = $1 AND status = 'active'
        ORDER BY created_at DESC
        "#,
        device_id,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| ApiError::Dependency(format!("Failed to fetch device sessions: {}", e)))?;

    Ok(sessions)
}

/// Terminates a session (marks as terminated, not deleted)
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `session_id` - Session UUID to terminate
///
/// # Returns
/// * `Ok(())` - Session terminated successfully
/// * `Err(ApiError::NotFound)` - Session not found
/// * `Err(ApiError::State)` - Session already terminated or expired
/// * `Err(ApiError::Dependency)` - Database error
pub async fn terminate_session(pool: &PgPool, session_id: Uuid) -> Result<(), ApiError> {
    let rows_affected = sqlx::query!(
        r#"
        UPDATE sessions
        SET status = 'terminated', terminated_at = NOW()
        WHERE session_id = $1 AND status = 'active'
        "#,
        session_id,
    )
    .execute(pool)
    .await
    .map_err(|e| ApiError::Dependency(format!("Failed to terminate session: {}", e)))?
    .rows_affected();

    if rows_affected == 0 {
        // Check if session exists but is not active
        let session = get_session(pool, session_id).await;
        match session {
            Ok(s) if s.status != SessionStatus::Active => {
                Err(ApiError::State(format!(
                    "Session is already {}",
                    s.status
                )))
            }
            Err(ApiError::NotFound(_)) => Err(ApiError::NotFound("Session not found".to_string())),
            _ => Err(ApiError::Dependency(
                "Failed to terminate session (unknown state)".to_string(),
            )),
        }
    } else {
        Ok(())
    }
}

/// Cleans up expired sessions older than retention period
///
/// This function should be called periodically (e.g., hourly) to remove
/// expired sessions from the database. Default retention is 24 hours.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `retention_hours` - Hours to keep expired sessions (default: 24)
///
/// # Returns
/// * `Ok(u64)` - Number of sessions deleted
/// * `Err(ApiError::Dependency)` - Database error
///
/// # Example
/// ```no_run
/// use honeylink_control_plane::db::sessions::cleanup_expired_sessions;
///
/// # async fn example(pool: sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
/// let deleted = cleanup_expired_sessions(&pool, 24).await?;
/// println!("Deleted {} expired sessions", deleted);
/// # Ok(())
/// # }
/// ```
pub async fn cleanup_expired_sessions(
    pool: &PgPool,
    retention_hours: i32,
) -> Result<u64, ApiError> {
    let result = sqlx::query!(
        r#"
        SELECT cleanup_expired_sessions($1) AS count
        "#,
        retention_hours,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| ApiError::Dependency(format!("Failed to cleanup expired sessions: {}", e)))?;

    Ok(result.count.unwrap_or(0) as u64)
}

/// Marks expired sessions as expired (manual trigger for auto_expire_sessions trigger)
///
/// This function manually runs the expiration logic that normally runs via trigger.
/// Useful for batch expiration without individual UPDATEs.
///
/// # Arguments
/// * `pool` - Database connection pool
///
/// # Returns
/// * `Ok(u64)` - Number of sessions expired
/// * `Err(ApiError::Dependency)` - Database error
pub async fn expire_outdated_sessions(pool: &PgPool) -> Result<u64, ApiError> {
    let rows_affected = sqlx::query!(
        r#"
        UPDATE sessions
        SET status = 'expired'
        WHERE status = 'active' AND expires_at < NOW()
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| ApiError::Dependency(format!("Failed to expire sessions: {}", e)))?
    .rows_affected();

    Ok(rows_affected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_status_display() {
        assert_eq!(SessionStatus::Active.to_string(), "active");
        assert_eq!(SessionStatus::Expired.to_string(), "expired");
        assert_eq!(SessionStatus::Terminated.to_string(), "terminated");
    }

    #[test]
    fn test_session_status_serialization() {
        let status = SessionStatus::Active;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, r#""active""#);

        let deserialized: SessionStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, SessionStatus::Active);
    }
}
