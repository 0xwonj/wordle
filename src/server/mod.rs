use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use rustls::crypto::ring;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::{self, TraceLayer};
use tracing::Level;

use crate::api;
use crate::config::Config;
use crate::state::AppState;

// Export the repository module
pub mod repository;

/// Initialize and run the server with the given configuration
pub async fn run(state: Arc<AppState>, config: &Config) -> Result<()> {
    // Build our application with routes
    let app = build_router(state);

    // Run the server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Listening on {}", addr);

    if config.tls.enabled {
        run_tls_server(app, addr, config).await?;
    } else {
        run_http_server(app, addr).await?;
    }

    Ok(())
}

/// Configure the application router with middleware
fn build_router(state: Arc<AppState>) -> Router {
    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .merge(api::router(state.clone()))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(cors)
}

/// Run the server with TLS enabled
async fn run_tls_server(app: Router, addr: SocketAddr, config: &Config) -> Result<()> {
    tracing::info!("TLS is enabled, using HTTPS with HTTP/2 support");
    tracing::info!("Loading certificates from: {:?}", config.tls.cert_file);
    tracing::info!("Loading key from: {:?}", config.tls.key_file);

    // Initialize rustls CryptoProvider - required in rustls 0.23+
    // Using let _ to ignore the error if it's already installed
    let _ = ring::default_provider().install_default();

    // Configure with TLS - using axum_server
    let rustls_config = RustlsConfig::from_pem_file(&config.tls.cert_file, &config.tls.key_file)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to load TLS config: {}", e))?;

    // Run server with TLS and HTTP/2 support
    axum_server::bind_rustls(addr, rustls_config)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

/// Run the server without TLS
async fn run_http_server(app: Router, addr: SocketAddr) -> Result<()> {
    tracing::warn!("TLS is disabled - running without HTTPS or HTTP/2 support");
    tracing::warn!("HTTP/2 requires TLS in most browsers");

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
