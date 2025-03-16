use std::sync::Arc;

use anyhow::Result;

// Import from our library instead of declaring modules
use wordle::{
    config::Config, init_logging, load_env, server, server::repository::init_repositories,
    state::AppState,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    load_env();

    // Initialize tracing
    init_logging();

    // Load configuration
    let config = Config::load()?;

    // Log configuration info
    tracing::info!("Starting Wordle game service on port {}", config.port);
    tracing::info!("JWT auth type: {}", config.jwt.auth_type);
    tracing::info!("JWT issuer: {}", config.jwt.issuer);
    tracing::info!("JWT audience: {}", config.jwt.audience);
    tracing::info!("TLS enabled: {}", config.tls.enabled);

    // Initialize repositories based on configuration
    let (game_repo, user_repo) = init_repositories(&config).await?;

    // Create application state with repositories
    let state = Arc::new(AppState::with_repositories(
        game_repo,
        user_repo,
        &config.jwt,
    )?);

    // Run the server
    server::run(state, &config).await?;

    Ok(())
}
