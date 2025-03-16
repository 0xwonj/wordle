use axum::{
    extract::{Request, State},
    http::header,
    middleware::{self, Next},
    response::Response,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::error::AuthError;
use crate::core::AppState;

/// Authentication middleware for protected routes
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    // Extract the token from the Authorization header
    let token = request
        .headers()
        .get(header::AUTHORIZATION)
        .ok_or(AuthError::Unauthorized)?
        .to_str()
        .map_err(|_| AuthError::Unauthorized)?;

    // Validate Bearer prefix
    if !token.starts_with("Bearer ") {
        return Err(AuthError::Unauthorized);
    }
    let token = &token[7..]; // Skip "Bearer " prefix

    // Verify the token
    let claims = state.jwt_auth().verify(token)?;

    // Extract user ID from claims
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AuthError::Unauthorized)?;

    // Add user ID and claims to request extensions
    request.extensions_mut().insert(user_id);
    request.extensions_mut().insert(claims);

    // Continue with the request
    Ok(next.run(request).await)
}

/// Create authentication middleware layer for a Router
///
/// This is a convenience function that can be used to protect
/// entire routers with authentication without manually applying
/// middleware to each individual route.
///
/// # Example
///
/// ```rust
/// let app = Router::new()
///     .route("/public", get(public_handler))
///     .nest(
///         "/protected",
///         Router::new()
///             .route("/profile", get(profile_handler))
///             .layer(require_auth(state.clone()))
///     )
/// ```
pub fn require_auth(state: Arc<AppState>) -> impl Clone {
    middleware::from_fn_with_state::<_, _, Request>(state, auth_middleware)
}
