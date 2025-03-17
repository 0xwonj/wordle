pub mod database;
pub mod error;
pub mod memory;

use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::models::User;
use crate::common::config::Config;
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

// Re-export database implementations
#[cfg(feature = "database")]
pub use database::postgres::game::PostgresGameRepository;
#[cfg(feature = "database")]
pub use database::postgres::user::PostgresUserRepository;
#[cfg(feature = "database")]
pub use database::postgres::{PostgresConfig, PostgresConnection};

/// Initialize repositories based on configuration
#[cfg(feature = "database")]
pub async fn init_repositories(
    config: &Config,
) -> anyhow::Result<(
    Arc<dyn GameRepositoryTrait + Send + Sync>,
    Arc<dyn UserRepositoryTrait + Send + Sync>,
)> {
    use crate::repository::{
        InMemoryGameRepository, InMemoryUserRepository, PostgresConfig, PostgresConnection,
        PostgresGameRepository, PostgresUserRepository,
    };

    if config.database.enabled {
        tracing::info!("Using PostgreSQL database for persistence");

        // Initialize PostgreSQL connection
        let db_config = PostgresConfig::new(&config.database.url);
        let pool = db_config.create_pool().await?;
        let connection = PostgresConnection::new(pool);

        // Create repositories
        let game_repo = Arc::new(PostgresGameRepository::new(connection.clone()))
            as Arc<dyn GameRepositoryTrait + Send + Sync>;
        let user_repo = Arc::new(PostgresUserRepository::new(connection))
            as Arc<dyn UserRepositoryTrait + Send + Sync>;

        Ok((game_repo, user_repo))
    } else {
        tracing::info!(
            "Database feature is enabled but database is disabled in config - using in-memory storage"
        );

        let game_repo =
            Arc::new(InMemoryGameRepository::new()) as Arc<dyn GameRepositoryTrait + Send + Sync>;
        let user_repo =
            Arc::new(InMemoryUserRepository::new()) as Arc<dyn UserRepositoryTrait + Send + Sync>;

        Ok((game_repo, user_repo))
    }
}

/// Initialize repositories (in-memory fallback when database feature is not enabled)
#[cfg(not(feature = "database"))]
pub async fn init_repositories(
    _config: &Config,
) -> anyhow::Result<(
    Arc<dyn GameRepositoryTrait + Send + Sync>,
    Arc<dyn UserRepositoryTrait + Send + Sync>,
)> {
    use crate::repository::memory::{InMemoryGameRepository, InMemoryUserRepository};

    tracing::info!("Database feature not enabled - using in-memory storage");

    let game_repo =
        Arc::new(InMemoryGameRepository::new()) as Arc<dyn GameRepositoryTrait + Send + Sync>;
    let user_repo =
        Arc::new(InMemoryUserRepository::new()) as Arc<dyn UserRepositoryTrait + Send + Sync>;

    Ok((game_repo, user_repo))
}
