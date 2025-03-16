use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// Authentication-specific error types
#[derive(Debug, Error)]
pub enum AuthError {
    /// JWT token is invalid or expired
    #[error("Invalid or expired JWT token")]
    JwtTokenInvalid,

    /// User is not authorized
    #[error("Unauthorized")]
    Unauthorized,

    /// Internal server error
    #[error("Internal server error: {0}")]
    InternalError(#[from] anyhow::Error),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::JwtTokenInvalid => (StatusCode::UNAUTHORIZED, self.to_string()),
            AuthError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            AuthError::InternalError(e) => {
                // Log the internal error
                tracing::error!("Internal server error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

/// Result type for authentication operations
pub type Result<T> = std::result::Result<T, AuthError>;
