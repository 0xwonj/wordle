use chrono::{DateTime, Local};
use parking_lot::RwLock;
use std::sync::Arc;
use uuid::Uuid;

use crate::game::GameService;
use crate::models::Game;
use crate::repository::GameRepositoryTrait;
use crate::repository::error::RepositoryResult;

/// Game state that will be shared across routes
pub struct GameState {
    /// Repository for game data access
    game_repository: Arc<dyn GameRepositoryTrait + Send + Sync>,

    /// Last date check (for daily word refresh)
    last_date_check: RwLock<DateTime<Local>>,

    /// Game service for game logic
    game_service: GameService,
}

impl GameState {
    /// Create a new game state with provided repository
    pub fn new(game_repository: Arc<dyn GameRepositoryTrait + Send + Sync>) -> Self {
        Self {
            game_repository,
            last_date_check: RwLock::new(Local::now()),
            game_service: GameService::new(),
        }
    }

    /// Get the game repository
    pub fn game_repository(&self) -> &(dyn GameRepositoryTrait + Send + Sync) {
        self.game_repository.as_ref()
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
