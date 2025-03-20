use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::models::{Game, LetterResult};

#[derive(Debug, Serialize)]
pub struct GameResponse {
    pub id: Uuid,
    pub attempts_remaining: u8,
    pub completed: bool,
    pub won: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub word: Option<String>,
    pub guesses: Vec<GuessResponse>,
}

#[derive(Debug, Serialize, Clone)]
pub struct GuessResponse {
    pub word: String,
    pub results: Vec<LetterResult>,
}

#[derive(Debug, Deserialize)]
pub struct CreateGameRequest {}

#[derive(Debug, Deserialize)]
pub struct GuessRequest {
    pub word: String,
}

impl From<Game> for GameResponse {
    fn from(game: Game) -> Self {
        // Only expose the secret word if the game is completed
        let word = game.completed.then_some(game.word.clone());

        // Convert the guesses to GuessResponse
        let guesses = game
            .guesses
            .iter()
            .map(|g| GuessResponse {
                word: g.word.clone(),
                results: g.results.clone(),
            })
            .collect();

        Self {
            id: game.id,
            attempts_remaining: game.attempts_remaining(),
            completed: game.completed,
            won: game.won,
            word,
            guesses,
        }
    }
}
