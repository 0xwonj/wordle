use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;
use tracing;

use crate::repository::error::RepositoryError;

/// Game-specific error types
#[derive(Debug, Error)]
pub enum GameError {
    /// Game is already completed
    #[error("Game is already completed")]
    GameCompleted,

    /// Invalid word provided
    #[error("Invalid word: {0}")]
    InvalidWord(String),

    /// Game not found
    #[error("Game not found")]
    GameNotFound,

    /// Repository error
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

impl IntoResponse for GameError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Self::GameCompleted => (StatusCode::BAD_REQUEST, self.to_string()),
            Self::InvalidWord(msg) => (StatusCode::BAD_REQUEST, msg),
            Self::GameNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            Self::Repository(err) => {
                // Log the repository error
                tracing::error!("Repository error: {}", err);
                match err {
                    RepositoryError::NotFound => (StatusCode::NOT_FOUND, "Game not found".into()),
                    RepositoryError::DatabaseError(_) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Database error occurred".into(),
                    ),
                    RepositoryError::ConnectionError(_) => (
                        StatusCode::SERVICE_UNAVAILABLE,
                        "Service unavailable".into(),
                    ),
                    RepositoryError::SerializationError(_) => {
                        (StatusCode::INTERNAL_SERVER_ERROR, "Data error".into())
                    }
                    RepositoryError::LockError(_) => (
                        StatusCode::SERVICE_UNAVAILABLE,
                        "Temporary unavailable".into(),
                    ),
                    RepositoryError::Unsupported(_) => (
                        StatusCode::NOT_IMPLEMENTED,
                        "Operation not supported".into(),
                    ),
                }
            }
        };

        // Construct JSON response
        (status, Json(json!({ "error": error_message }))).into_response()
    }
}
