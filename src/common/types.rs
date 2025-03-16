use anyhow::Error;

/// Represents a result type that can return either a value or an error
pub type WordleResult<T> = std::result::Result<T, Error>;
