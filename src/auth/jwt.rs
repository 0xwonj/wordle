use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use time::OffsetDateTime;

use crate::auth::error::{AuthError, Result};
use crate::auth::models::Claims;
use crate::common::config::JwtConfig;

/// JWT authentication service
pub struct JwtAuth {
    /// Key for JWT signature verification
    decoding_key: DecodingKey,

    /// Algorithm to use
    algorithm: Algorithm,

    /// Issuer setting
    issuer: String,

    /// Audience setting
    audience: String,
}

impl JwtAuth {
    /// Create a new JWT authentication service
    pub fn new(config: &JwtConfig) -> Result<Self> {
        // Select algorithm
        let algorithm = match config.auth_type.as_str() {
            "secret" => Algorithm::HS256,
            "rsa" => Algorithm::RS256,
            "ed25519" => Algorithm::EdDSA,
            _ => {
                return Err(AuthError::InternalError(anyhow::anyhow!(
                    "Unsupported JWT auth type"
                )));
            }
        };

        // Create decoding key
        let decoding_key = match config.auth_type.as_str() {
            "secret" => DecodingKey::from_secret(config.public_key.as_bytes()),
            "rsa" => DecodingKey::from_rsa_pem(config.public_key.as_bytes())
                .map_err(|e| AuthError::InternalError(anyhow::anyhow!("Invalid RSA key: {}", e)))?,
            "ed25519" => DecodingKey::from_ed_pem(config.public_key.as_bytes()).map_err(|e| {
                AuthError::InternalError(anyhow::anyhow!("Invalid Ed25519 key: {}", e))
            })?,
            _ => {
                return Err(AuthError::InternalError(anyhow::anyhow!(
                    "Unsupported JWT auth type"
                )));
            }
        };

        Ok(Self {
            decoding_key,
            algorithm,
            issuer: config.issuer.clone(),
            audience: config.audience.clone(),
        })
    }

    /// Verify JWT token
    pub fn verify(&self, token: &str) -> Result<Claims> {
        // Validation settings
        let mut validation = Validation::new(self.algorithm);

        // Set required claims
        validation.set_required_spec_claims(&["exp", "sub", "iat"]);

        // Verify issuer (optional)
        if !self.issuer.is_empty() {
            validation.set_issuer(&[&self.issuer]);
        }

        // Verify audience (optional)
        if !self.audience.is_empty() {
            validation.set_audience(&[&self.audience]);
        }

        // Decode and verify token
        let token_data = decode::<Claims>(token, &self.decoding_key, &validation).map_err(|e| {
            tracing::debug!("JWT verification failed: {:?}", e);
            AuthError::JwtTokenInvalid
        })?;

        // Additional verification: check token expiration
        let now = OffsetDateTime::now_utc().unix_timestamp() as usize;
        if now > token_data.claims.exp {
            tracing::debug!("JWT token expired");
            return Err(AuthError::JwtTokenInvalid);
        }

        Ok(token_data.claims)
    }
}
