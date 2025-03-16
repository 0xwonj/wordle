use async_trait::async_trait;
use parking_lot::RwLock;
use std::collections::HashMap;
use uuid::Uuid;

use crate::game::models::Game;
use crate::repository::error::{RepositoryError, RepositoryResult};
use crate::repository::GameRepositoryTrait;

/// In-memory implementation of game repository
/// Useful for testing and development environments
#[derive(Debug, Default)]
pub struct InMemoryGameRepository {
    /// In-memory game storage, keyed by game ID
    games: RwLock<HashMap<Uuid, Game>>,
}

impl InMemoryGameRepository {
    /// Create a new in-memory game repository
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl GameRepositoryTrait for InMemoryGameRepository {
    /// Get a game by ID
    async fn get_game(&self, id: &Uuid) -> RepositoryResult<Game> {
        let games = self.games.read();

        games.get(id).cloned().ok_or(RepositoryError::NotFound)
    }

    /// Save a game
    async fn save_game(&self, game: Game) -> RepositoryResult<()> {
        let mut games = self.games.write();
        games.insert(game.id, game);
        Ok(())
    }

    /// Clear all games and return the count of cleared games
    async fn clear_all_games(&self) -> RepositoryResult<usize> {
        let mut games = self.games.write();
        let cleared_count = games.len();
        games.clear();

        Ok(cleared_count)
    }
}
