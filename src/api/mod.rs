use std::sync::Arc;

use axum::{
    middleware,
    routing::{get, post},
    Router,
};

pub mod handlers;

use crate::auth::auth_middleware;
use crate::state::AppState;

/// Configure API routes
pub fn router(state: Arc<AppState>) -> Router {
    // Create health check route that doesn't need state
    let health_route = Router::new().route("/health", get(handlers::util::health_check));

    // Create protected game routes with auth
    let game_routes = Router::new()
        .route("/new", post(handlers::game::create_game))
        .route("/{id}", get(handlers::game::get_game))
        .route("/{id}/guess", post(handlers::game::make_guess))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state.clone());

    // Combine all routes
    Router::new()
        .nest(
            "/api",
            health_route.merge(Router::new().nest("/game", game_routes)),
        )
        .with_state(state)
}
