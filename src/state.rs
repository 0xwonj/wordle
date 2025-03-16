use chrono::{DateTime, Local};
use parking_lot::RwLock;
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::error::Result as AuthResult;
use crate::auth::jwt::JwtAuth;
use crate::config::JwtConfig;
use crate::game::GameService;
use crate::models::Game;
use crate::repository::error::RepositoryResult;
use crate::repository::{
    GameRepositoryTrait, InMemoryGameRepository, InMemoryUserRepository, UserRepositoryTrait,
};

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
    /// Create a new application state with default in-memory repositories
    pub fn new(jwt_config: &JwtConfig) -> AuthResult<Self> {
        Self::with_repositories(
            Arc::new(InMemoryGameRepository::default()),
            Arc::new(InMemoryUserRepository::default()),
            jwt_config,
        )
    }

    /// Create a new application state with custom repositories
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
            game_service: GameService::default(),
            last_date_check: RwLock::new(Local::now()),
        })
    }

    /// Get the current game for a user
    pub async fn get_current_user_game(&self, user_id: &Uuid) -> RepositoryResult<Option<Game>> {
        // Get user information
        let user = self.user_repository.get_user(user_id).await?;

        // Check user's current game ID and return the game if it exists
        match user.current_game_id {
            Some(game_id) => {
                let game = self.game_repository.get_game(&game_id).await?;
                Ok(Some(game))
            }
            None => Ok(None),
        }
    }

    /// Get reference to JWT authentication service
    pub fn jwt_auth(&self) -> &JwtAuth {
        &self.jwt_auth
    }

    /// Get reference to game repository
    pub fn game_repository(&self) -> &(dyn GameRepositoryTrait + Send + Sync) {
        &*self.game_repository
    }

    /// Get reference to user repository
    pub fn user_repository(&self) -> &(dyn UserRepositoryTrait + Send + Sync) {
        &*self.user_repository
    }

    /// Get reference to game service
    pub fn game_service(&self) -> &GameService {
        &self.game_service
    }

    /// Get reference to last date check
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

    /// Update a user's current game ID
    pub async fn update_user_game(&self, user_id: &Uuid, game_id: Uuid) -> RepositoryResult<bool> {
        self.user_repository
            .update_user_game(user_id, game_id)
            .await
    }

    /// Check if the day has changed and clear outdated games
    pub async fn check_and_update_date(&self) -> RepositoryResult<()> {
        let now = Local::now();
        let date_changed = {
            // Scope the read lock to ensure it's dropped before we make any changes
            let current_date = self.last_date_check.read();
            now.date_naive() != current_date.date_naive()
        };

        if date_changed {
            // We need to update the date and reset games
            {
                // Update the date immediately to prevent race conditions
                let mut current_date = self.last_date_check.write();
                let prev_date = current_date.date_naive();
                let new_date = now.date_naive();

                tracing::info!(
                    "Day change detected: {} -> {}. Resetting all game states.",
                    prev_date,
                    new_date
                );

                *current_date = now;
            }

            // Reset user game states and clear all games
            let updated_user_count = self.user_repository.reset_all_users_current_game().await?;
            let cleared_game_count = self.game_repository.clear_all_games().await?;

            // Log the results of the cleanup
            tracing::info!(
                "Day change completed: Reset {} user states and cleared {} games",
                updated_user_count,
                cleared_game_count
            );

            // Force refresh of the daily word
            let new_word = self.game_service.select_daily_word();
            tracing::info!("New daily word selected (not shown in logs for security)");
            tracing::debug!("Debug only - New word: {}", new_word);
        }

        Ok(())
    }
}
