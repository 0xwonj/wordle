// Repository module for data access

pub mod database;
pub mod error;
pub mod game;
pub mod memory;
pub mod user;

use async_trait::async_trait;
use uuid::Uuid;

use crate::auth::models::User;
use crate::game::models::Game;
use error::RepositoryResult;

/// Repository trait for game data access
#[async_trait]
pub trait GameRepositoryTrait: Send + Sync {
    /// Get a game by ID
    async fn get_game(&self, id: &Uuid) -> RepositoryResult<Game>;

    /// Save a game
    async fn save_game(&self, game: Game) -> RepositoryResult<()>;

    /// Clear all games and return the count of cleared games
    async fn clear_all_games(&self) -> RepositoryResult<usize>;
}

/// Repository trait for user data access
#[async_trait]
pub trait UserRepositoryTrait: Send + Sync {
    /// Get a user by ID
    async fn get_user(&self, id: &Uuid) -> RepositoryResult<User>;

    /// Save a user
    async fn save_user(&self, user: User) -> RepositoryResult<()>;

    /// Update a user's current game ID
    async fn update_user_game(&self, user_id: &Uuid, game_id: Uuid) -> RepositoryResult<bool>;

    /// Reset all users' current game IDs and return the count of updated users
    async fn reset_all_users_current_game(&self) -> RepositoryResult<usize>;
}

// Re-export memory implementations for backwards compatibility
pub use memory::game::InMemoryGameRepository;
pub use memory::user::InMemoryUserRepository;

// Re-export database implementations
#[cfg(feature = "database")]
pub use database::postgres::game::PostgresGameRepository;
#[cfg(feature = "database")]
pub use database::postgres::user::PostgresUserRepository;
#[cfg(feature = "database")]
pub use database::postgres::{PostgresConfig, PostgresConnection};
