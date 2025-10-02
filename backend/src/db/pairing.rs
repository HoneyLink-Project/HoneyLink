// Pairing code database operations

use crate::error::ApiError;
use crate::types::DeviceId;
use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use sqlx::PgPool;

/// Pairing code record in the database
#[derive(Debug, Clone)]
pub struct PairingCode {
    pub pairing_code: String,
    pub device_id: String,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Generates a secure pairing code in format "XXXX-XXXX-XXXX"
/// Uses charset without ambiguous characters (no 0/O, 1/I/l)
pub fn generate_pairing_code() -> String {
    const CHARSET: &[u8] = b"23456789ABCDEFGHJKLMNPQRSTUVWXYZ";
    let mut rng = rand::thread_rng();

    let parts: Vec<String> = (0..3)
        .map(|_| {
            (0..4)
                .map(|_| {
                    let idx = rng.gen_range(0..CHARSET.len());
                    CHARSET[idx] as char
                })
                .collect()
        })
        .collect();

    parts.join("-")
}

/// Creates a new pairing code with 10-minute TTL
pub async fn create_pairing_code(
    pool: &PgPool,
    device_id: &DeviceId,
) -> Result<(String, DateTime<Utc>), ApiError> {
    let pairing_code = generate_pairing_code();
    let expires_at = Utc::now() + Duration::minutes(10);

    sqlx::query!(
        r#"
        INSERT INTO pairing_codes (pairing_code, device_id, expires_at)
        VALUES ($1, $2, $3)
        "#,
        pairing_code,
        device_id.0,
        expires_at
    )
    .execute(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(db_err) if db_err.constraint() == Some("pairing_codes_pkey") => {
            // Extremely unlikely collision, retry would be better in production
            ApiError::Internal("Pairing code collision".to_string())
        }
        _ => ApiError::Dependency(format!("Database error: {}", e)),
    })?;

    Ok((pairing_code, expires_at))
}

/// Validates and consumes a pairing code
pub async fn validate_and_consume_pairing_code(
    pool: &PgPool,
    device_id: &DeviceId,
    pairing_code: &str,
) -> Result<(), ApiError> {
    // Check if pairing code exists, matches device_id, not expired, and not used
    let result = sqlx::query!(
        r#"
        SELECT expires_at, used_at
        FROM pairing_codes
        WHERE pairing_code = $1 AND device_id = $2
        "#,
        pairing_code,
        device_id.0
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| ApiError::Dependency(format!("Database error: {}", e)))?;

    let record = result.ok_or_else(|| {
        ApiError::Validation("Invalid pairing code".to_string())
    })?;

    // Check if already used
    if record.used_at.is_some() {
        return Err(ApiError::State("Pairing code already used".to_string()));
    }

    // Check if expired
    if record.expires_at < Utc::now() {
        return Err(ApiError::State("Pairing code expired".to_string()));
    }

    // Mark as used
    sqlx::query!(
        r#"
        UPDATE pairing_codes
        SET used_at = NOW()
        WHERE pairing_code = $1
        "#,
        pairing_code
    )
    .execute(pool)
    .await
    .map_err(|e| ApiError::Dependency(format!("Database error: {}", e)))?;

    Ok(())
}

/// Cleans up expired pairing codes (should be run periodically)
pub async fn cleanup_expired_pairing_codes(pool: &PgPool) -> Result<u64, ApiError> {
    let result = sqlx::query!(
        "DELETE FROM pairing_codes WHERE expires_at < NOW() - INTERVAL '1 hour'"
    )
    .execute(pool)
    .await
    .map_err(|e| ApiError::Dependency(format!("Database error: {}", e)))?;

    Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_pairing_code_format() {
        let code = generate_pairing_code();
        assert_eq!(code.len(), 14); // "XXXX-XXXX-XXXX" = 14 chars
        assert_eq!(code.matches('-').count(), 2);

        let parts: Vec<&str> = code.split('-').collect();
        assert_eq!(parts.len(), 3);
        assert!(parts.iter().all(|p| p.len() == 4));
    }

    #[test]
    fn test_generate_pairing_code_charset() {
        let code = generate_pairing_code();
        let code_no_dash = code.replace('-', "");

        // Should not contain ambiguous characters
        assert!(!code_no_dash.contains('0'));
        assert!(!code_no_dash.contains('O'));
        assert!(!code_no_dash.contains('1'));
        assert!(!code_no_dash.contains('I'));
        assert!(!code_no_dash.contains('l'));
    }

    #[test]
    fn test_generate_pairing_code_uniqueness() {
        let codes: std::collections::HashSet<_> = (0..100)
            .map(|_| generate_pairing_code())
            .collect();

        // Should generate unique codes (very high probability)
        assert_eq!(codes.len(), 100);
    }
}
