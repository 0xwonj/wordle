pub mod api;
pub mod auth;
pub mod config;
pub mod game;
pub mod models;
pub mod repository;
pub mod server;
pub mod state;
pub mod utils;

// Re-export common traits and types
pub use anyhow::{Error, Result};

// Constants that might be used across binaries
pub const APP_NAME: &str = "Wordle";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

// Common error and result types
/// Represents a result type that can return either a value or an error
pub type WordleResult<T> = std::result::Result<T, Error>;

/// Common API-related errors
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

// Common initialization functions
/// Initialize logging with sensible defaults
pub fn init_logging() {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();
}

/// Load environment variables from .env file
pub fn load_env() {
    dotenv::dotenv().ok();
}

/// Utility function to get current timestamp in seconds
pub fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}
