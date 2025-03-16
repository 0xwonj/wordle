use std::sync::Arc;

use axum::{
    extract::{FromRequestParts, State},
    http::{Extensions, header, request::Parts},
};
use uuid::Uuid;

use crate::auth::error::AuthError;
use crate::auth::models::Claims;
use crate::core::AppState;

/// Auth extractor that provides the authenticated user ID and claims
#[derive(Debug, Clone)]
pub struct Auth {
    /// The authenticated user's ID
    pub user_id: Uuid,
    /// The JWT claims
    pub claims: Claims,
}

impl Auth {
    /// Try to extract Auth from request extensions
    ///
    /// This is a performance optimization when used with auth middleware
    pub fn try_from_extensions(extensions: &Extensions) -> Option<Self> {
        let user_id = extensions.get::<Uuid>().copied()?;
        let claims = extensions.get::<Claims>().cloned()?;
        Some(Self { user_id, claims })
    }
}

impl<S> FromRequestParts<S> for Auth
where
    S: Send + Sync,
    State<Arc<AppState>>: FromRequestParts<S>,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // First try to get from extensions (if auth middleware was used)
        if let Some(auth) = Self::try_from_extensions(&parts.extensions) {
            return Ok(auth);
        }

        // Otherwise extract the AppState and perform the full verification
        let State(app_state) = State::<Arc<AppState>>::from_request_parts(parts, state)
            .await
            .map_err(|_| AuthError::InternalError(anyhow::anyhow!("Failed to extract state")))?;

        // Extract the token from the Authorization header
        let token = parts
            .headers
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
        let claims = app_state.jwt_auth().verify(token)?;

        // Extract user ID from claims
        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AuthError::Unauthorized)?;

        Ok(Self { user_id, claims })
    }
}

/// A simplified extractor that only provides the user ID
#[derive(Debug, Clone, Copy)]
pub struct AuthUserId(pub Uuid);

impl AuthUserId {
    /// Try to extract user ID from request extensions
    ///
    /// This is a performance optimization when used with auth middleware
    pub fn try_from_extensions(extensions: &Extensions) -> Option<Self> {
        extensions.get::<Uuid>().map(|&id| Self(id))
    }
}

impl<S> FromRequestParts<S> for AuthUserId
where
    S: Send + Sync,
    Auth: FromRequestParts<S, Rejection = AuthError>,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // First try to get from extensions (if auth middleware was used)
        if let Some(user_id) = Self::try_from_extensions(&parts.extensions) {
            return Ok(user_id);
        }

        // Otherwise use the Auth extractor
        let auth = Auth::from_request_parts(parts, state).await?;
        Ok(Self(auth.user_id))
    }
}
