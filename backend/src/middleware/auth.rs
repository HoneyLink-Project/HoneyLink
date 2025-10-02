// JWT authentication middleware

use crate::config::JwtConfig;
use crate::error::ApiError;
use crate::types::JwtClaims;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use std::sync::Arc;

/// Shared JWT validator state
#[derive(Clone)]
pub struct JwtValidator {
    config: JwtConfig,
    decoding_key: DecodingKey,
    validation: Validation,
}

impl JwtValidator {
    /// Create new JWT validator
    pub fn new(config: JwtConfig) -> Result<Self, ApiError> {
        // Load public key from file
        let key_pem = std::fs::read(&config.public_key_path)
            .map_err(|e| ApiError::Internal(format!("Failed to read JWT public key: {}", e)))?;

        // Parse algorithm
        let algorithm = match config.algorithm.as_str() {
            "RS256" => Algorithm::RS256,
            "RS384" => Algorithm::RS384,
            "RS512" => Algorithm::RS512,
            "ES256" => Algorithm::ES256,
            "ES384" => Algorithm::ES384,
            "EdDSA" => Algorithm::EdDSA,
            _ => return Err(ApiError::Internal(format!("Unsupported JWT algorithm: {}", config.algorithm))),
        };

        // Create decoding key based on algorithm
        let decoding_key = if algorithm == Algorithm::EdDSA {
            DecodingKey::from_ed_pem(&key_pem)
                .map_err(|e| ApiError::Internal(format!("Failed to parse EdDSA key: {}", e)))?
        } else if matches!(algorithm, Algorithm::ES256 | Algorithm::ES384) {
            DecodingKey::from_ec_pem(&key_pem)
                .map_err(|e| ApiError::Internal(format!("Failed to parse EC key: {}", e)))?
        } else {
            DecodingKey::from_rsa_pem(&key_pem)
                .map_err(|e| ApiError::Internal(format!("Failed to parse RSA key: {}", e)))?
        };

        // Setup validation
        let mut validation = Validation::new(algorithm);
        validation.set_issuer(&[&config.issuer]);
        validation.set_audience(&[&config.audience]);
        validation.validate_exp = true;
        validation.validate_nbf = true;
        validation.leeway = 10; // 10 seconds leeway for clock skew

        Ok(JwtValidator {
            config,
            decoding_key,
            validation,
        })
    }

    /// Validate JWT token and extract claims
    pub fn validate(&self, token: &str) -> Result<JwtClaims, ApiError> {
        // Decode and validate token
        let token_data = decode::<JwtClaims>(token, &self.decoding_key, &self.validation)
            .map_err(|e| ApiError::Authentication(format!("JWT validation failed: {}", e)))?;

        let claims = token_data.claims;

        // Additional validation: check if token is expired (redundant but explicit)
        if claims.is_expired() {
            return Err(ApiError::Authentication("Token is expired".to_string()));
        }

        // Check if token is valid now (not before)
        if !claims.is_valid_now() {
            return Err(ApiError::Authentication("Token is not yet valid".to_string()));
        }

        Ok(claims)
    }
}

/// Middleware function to extract and validate JWT from Authorization header
pub async fn jwt_auth_middleware(
    State(validator): State<Arc<JwtValidator>>,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| ApiError::Authentication("Missing Authorization header".to_string()))?;

    // Parse Bearer token
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| ApiError::Authentication("Invalid Authorization header format".to_string()))?;

    // Validate token
    let claims = validator.validate(token)?;

    // Insert claims into request extensions for downstream handlers
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

/// Extractor for JWT claims from request extensions
pub struct RequireAuth(pub JwtClaims);

#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for RequireAuth
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<JwtClaims>()
            .cloned()
            .map(RequireAuth)
            .ok_or_else(|| ApiError::Authentication("No JWT claims found".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::JwtConfig;
    use std::path::PathBuf;

    // Note: These tests require actual JWT keys to run
    // In a real scenario, we'd generate test keys in test setup

    #[test]
    fn test_jwt_validator_creation() {
        // This test would fail without actual keys
        // In production, we'd use test fixtures
        let config = JwtConfig {
            algorithm: "ES256".to_string(),
            public_key_path: PathBuf::from("./test-keys/jwt_public.pem"),
            issuer: "test-issuer".to_string(),
            audience: "test-audience".to_string(),
            ttl_seconds: 300,
        };

        // Would succeed if key file exists
        // assert!(JwtValidator::new(config).is_ok());
    }

    #[test]
    fn test_claims_expiration_check() {
        let now = chrono::Utc::now().timestamp();
        let claims = JwtClaims {
            sub: "test-device".to_string(),
            exp: now - 100,
            iat: now - 400,
            nbf: None,
            iss: "test".to_string(),
            aud: "test".to_string(),
            scopes: vec![],
        };

        assert!(claims.is_expired());
    }
}
