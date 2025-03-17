pub mod api;
pub mod auth;
pub mod common;
pub mod game;
pub mod models;
pub mod repository;

// Re-export common traits and types
pub use anyhow::{Error, Result};
pub use api::error::ApiError;
pub use auth::AuthState;
pub use common::types::WordleResult;
pub use game::GameState;

// Constants that might be used across binaries
pub const APP_NAME: &str = "Wordle";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

// Re-export common utility functions from the common module
pub use common::utils::{current_timestamp, init_logging, load_env};
