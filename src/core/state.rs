use chrono::{DateTime, Local};
use parking_lot::RwLock;
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::error::Result as AuthResult;
use crate::auth::jwt::JwtAuth;
use crate::common::config::JwtConfig;
use crate::game::GameService;
use crate::models::Game;
use crate::repository::error::RepositoryResult;
use crate::repository::memory::{InMemoryGameRepository, InMemoryUserRepository};
use crate::repository::{GameRepositoryTrait, UserRepositoryTrait};

/// Application state that will be shared across all routes
pub struct AppState {
    /// Repository for game data access
    game_repository: Arc<dyn GameRepositoryTrait + Send + Sync>,

    /// Repository for user data access
    user_repository: Arc<dyn UserRepositoryTrait + Send + Sync>,

    /// JWT authentication service
    jwt_auth: JwtAuth,

    /// Last date check (for daily word refresh)
    last_date_check: RwLock<DateTime<Local>>,

    /// Game service for game logic
    game_service: GameService,
}

impl AppState {
    /// Create a new application state with in-memory repositories
    pub fn new(jwt_config: &JwtConfig) -> AuthResult<Self> {
        let game_repository = Arc::new(InMemoryGameRepository::new());
        let user_repository = Arc::new(InMemoryUserRepository::new());

        Self::with_repositories(game_repository, user_repository, jwt_config)
    }

    /// Create a new application state with provided repositories
    pub fn with_repositories(
        game_repository: Arc<dyn GameRepositoryTrait + Send + Sync>,
        user_repository: Arc<dyn UserRepositoryTrait + Send + Sync>,
        jwt_config: &JwtConfig,
    ) -> AuthResult<Self> {
        let jwt_auth = JwtAuth::new(jwt_config)?;

        Ok(Self {
            game_repository,
            user_repository,
            jwt_auth,
            last_date_check: RwLock::new(Local::now()),
            game_service: GameService::new(),
        })
    }

    /// Get the current game for a user
    pub async fn get_current_user_game(&self, user_id: &Uuid) -> RepositoryResult<Option<Game>> {
        let user = self.user_repository.get_user(user_id).await?;

        match user.current_game_id {
            Some(game_id) => {
                let game = self.game_repository.get_game(&game_id).await?;
                Ok(Some(game))
            }
            None => Ok(None),
        }
    }

    /// Get the JWT authentication service
    pub fn jwt_auth(&self) -> &JwtAuth {
        &self.jwt_auth
    }

    /// Get the game repository
    pub fn game_repository(&self) -> &(dyn GameRepositoryTrait + Send + Sync) {
        self.game_repository.as_ref()
    }

    /// Get the user repository
    pub fn user_repository(&self) -> &(dyn UserRepositoryTrait + Send + Sync) {
        self.user_repository.as_ref()
    }

    /// Get the game service
    pub fn game_service(&self) -> &GameService {
        &self.game_service
    }

    /// Get the last date check lock
    pub fn last_date_check(&self) -> &RwLock<DateTime<Local>> {
        &self.last_date_check
    }

    /// Get a game by ID
    pub async fn get_game(&self, id: &Uuid) -> RepositoryResult<Game> {
        self.game_repository.get_game(id).await
    }

    /// Save a game
    pub async fn save_game(&self, game: Game) -> RepositoryResult<()> {
        self.game_repository.save_game(game).await
    }

    /// Get a user by ID
    pub async fn get_user(&self, id: &Uuid) -> RepositoryResult<crate::auth::models::User> {
        self.user_repository.get_user(id).await
    }

    /// Save a user
    pub async fn save_user(&self, user: crate::auth::models::User) -> RepositoryResult<()> {
        self.user_repository.save_user(user).await
    }

    /// Update a user's current game
    pub async fn update_user_game(&self, user_id: &Uuid, game_id: Uuid) -> RepositoryResult<bool> {
        let mut user = self.user_repository.get_user(user_id).await?;
        user.current_game_id = Some(game_id);
        self.user_repository.save_user(user).await?;
        Ok(true)
    }

    /// Check if the date has changed and update the daily word if necessary
    pub async fn check_and_update_date(&self) -> RepositoryResult<()> {
        let now = Local::now();
        let mut last_check = self.last_date_check.write();

        // If the date has changed, update the daily word
        if now.date_naive() != last_check.date_naive() {
            // In a real app, we would update the daily word here
            *last_check = now;
        }

        Ok(())
    }
}
