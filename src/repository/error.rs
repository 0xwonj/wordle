/// Error types for repository operations
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Item not found")]
    NotFound,

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Data serialization error: {0}")]
    SerializationError(String),

    #[error("Lock acquisition error: {0}")]
    LockError(String),

    #[error("Operation not supported: {0}")]
    Unsupported(String),
}

/// Shorthand for repository operation results
pub type RepositoryResult<T> = Result<T, RepositoryError>;

/// Helper function to convert a lock error to a RepositoryError
pub fn lock_error<T, E: std::fmt::Display>(e: E) -> RepositoryResult<T> {
    Err(RepositoryError::LockError(e.to_string()))
}

/// Helper function for NotFound errors
pub fn not_found<T>() -> RepositoryResult<T> {
    Err(RepositoryError::NotFound)
}
