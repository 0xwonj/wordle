/// Database-backed repository implementations
/// This module is structured to enable easy addition of new database providers
// PostgreSQL implementation
pub mod postgres;

// Re-export the concrete repository implementations
#[cfg(feature = "database")]
pub use self::postgres::game::PostgresGameRepository;
#[cfg(feature = "database")]
pub use self::postgres::user::PostgresUserRepository;
