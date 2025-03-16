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
