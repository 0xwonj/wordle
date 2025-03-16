use std::sync::Arc;

use crate::config::Config;
use crate::repository::{GameRepositoryTrait, UserRepositoryTrait};

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
        tracing::info!("Database feature is enabled but database is disabled in config - using in-memory storage");

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
    use crate::repository::{InMemoryGameRepository, InMemoryUserRepository};

    tracing::info!("Database feature not enabled - using in-memory storage");

    let game_repo =
        Arc::new(InMemoryGameRepository::new()) as Arc<dyn GameRepositoryTrait + Send + Sync>;
    let user_repo =
        Arc::new(InMemoryUserRepository::new()) as Arc<dyn UserRepositoryTrait + Send + Sync>;

    Ok((game_repo, user_repo))
}
