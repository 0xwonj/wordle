/// PostgreSQL repository implementations
pub mod game;
pub mod user;

#[cfg(feature = "database")]
use sqlx::PgPool;
#[cfg(feature = "database")]
use std::sync::Arc;

/// Database connection configuration for PostgreSQL
#[cfg(feature = "database")]
pub struct PostgresConfig {
    /// Database connection URL
    pub connection_url: String,

    /// Maximum number of connections in the pool
    pub max_connections: u32,

    /// Connection timeout in seconds
    pub connection_timeout: u64,
}

#[cfg(feature = "database")]
impl PostgresConfig {
    /// Create a new PostgreSQL configuration with sensible defaults
    pub fn new(connection_url: impl Into<String>) -> Self {
        Self {
            connection_url: connection_url.into(),
            max_connections: 5,
            connection_timeout: 30,
        }
    }

    /// Create a connection pool using this configuration
    pub async fn create_pool(&self) -> Result<PgPool, sqlx::Error> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(self.max_connections)
            .acquire_timeout(std::time::Duration::from_secs(self.connection_timeout))
            .connect(&self.connection_url)
            .await?;

        Ok(pool)
    }
}

/// Shared database connection for repository implementations
#[cfg(feature = "database")]
#[derive(Clone)]
pub struct PostgresConnection {
    /// Connection pool to PostgreSQL database
    pub pool: Arc<PgPool>,
}

#[cfg(feature = "database")]
impl PostgresConnection {
    /// Create a new PostgreSQL connection with the given pool
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool: Arc::new(pool),
        }
    }
}
