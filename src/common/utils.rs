/// Initialize logging with sensible defaults
pub fn init_logging() {
    // Get log level from environment or use default
    let log_level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(true)
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
