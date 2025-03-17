use std::sync::Arc;
use uuid::Uuid;

use crate::auth::error::Result as AuthResult;
use crate::auth::jwt::JwtAuth;
use crate::auth::models::User;
use crate::common::config::JwtConfig;
use crate::repository::UserRepositoryTrait;
use crate::repository::error::RepositoryError;
use crate::repository::error::RepositoryResult;

/// Auth state that will be shared across routes
pub struct AuthState {
    /// Repository for user data access
    user_repository: Arc<dyn UserRepositoryTrait + Send + Sync>,

    /// JWT authentication service
    jwt_auth: JwtAuth,
}

impl AuthState {
    /// Create a new auth state with provided repository
    pub fn new(
        user_repository: Arc<dyn UserRepositoryTrait + Send + Sync>,
        jwt_config: &JwtConfig,
    ) -> AuthResult<Self> {
        let jwt_auth = JwtAuth::new(jwt_config)?;

        Ok(Self {
            user_repository,
            jwt_auth,
        })
    }

    /// Get the JWT authentication service
    pub fn jwt_auth(&self) -> &JwtAuth {
        &self.jwt_auth
    }

    /// Get the user repository
    pub fn user_repository(&self) -> &(dyn UserRepositoryTrait + Send + Sync) {
        self.user_repository.as_ref()
    }

    /// Get a user by ID
    pub async fn get_user(&self, id: &Uuid) -> RepositoryResult<User> {
        self.user_repository.get_user(id).await
    }

    /// Save a user
    pub async fn save_user(&self, user: User) -> RepositoryResult<()> {
        self.user_repository.save_user(user).await
    }

    /// Update a user's current game
    pub async fn update_user_game(&self, user_id: &Uuid, game_id: Uuid) -> RepositoryResult<bool> {
        self.user_repository
            .update_user_game(user_id, game_id)
            .await
    }

    /// Get the current game ID for a user
    pub async fn get_current_user_game_id(&self, user_id: &Uuid) -> RepositoryResult<Option<Uuid>> {
        match self.user_repository.get_user(user_id).await {
            Ok(user) => Ok(user.current_game_id),
            Err(RepositoryError::NotFound) => Ok(None),
            Err(err) => Err(err),
        }
    }
}
