use std::sync::Arc;

use axum::{
    Router,
    extract::FromRef,
    middleware,
    routing::{get, post},
};

pub mod error;
pub mod handlers;
pub mod models;

use crate::auth::{AuthState, auth_middleware};
use crate::game::GameState;

// Public struct for route state
#[derive(Clone)]
pub struct AppState {
    pub game: Arc<GameState>,
    pub auth: Arc<AuthState>,
}

// Implement FromRef for AppState to allow extracting AuthState
impl FromRef<AppState> for Arc<AuthState> {
    fn from_ref(state: &AppState) -> Self {
        state.auth.clone()
    }
}

// Implement FromRef for AppState to allow extracting GameState
impl FromRef<AppState> for Arc<GameState> {
    fn from_ref(state: &AppState) -> Self {
        state.game.clone()
    }
}

/// Configure API routes
pub fn router(game_state: Arc<GameState>, auth_state: Arc<AuthState>) -> Router {
    // Create combined state for routes
    let route_state = AppState {
        game: game_state,
        auth: auth_state.clone(),
    };

    // Create health check route that doesn't need state
    let health_route = Router::new().route("/health", get(handlers::util::health_check));

    // Create protected game routes with auth
    let game_routes = Router::new()
        .route("/new", post(handlers::game::create_game))
        .route("/{id}", get(handlers::game::get_game))
        .route("/{id}/guess", post(handlers::game::make_guess))
        .layer(middleware::from_fn_with_state(auth_state, auth_middleware))
        .with_state(route_state);

    // Combine all routes
    Router::new()
        .nest("/api", health_route)
        .nest("/api/game", game_routes)
}
