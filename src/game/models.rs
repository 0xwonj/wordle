use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::error::GameError;

/// Game model for storing game state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    /// Unique game identifier
    pub id: Uuid,

    /// User ID who owns this game
    pub user_id: Uuid,

    /// The secret word to guess
    pub word: String,

    /// Maximum number of attempts allowed
    pub max_attempts: u8,

    /// List of guesses made so far
    pub guesses: Vec<Guess>,

    /// Whether the game is completed
    pub completed: bool,

    /// Whether the player won
    pub won: bool,

    /// When the game was created
    pub created_at: DateTime<Utc>,

    /// When the game was last updated
    pub updated_at: DateTime<Utc>,
}

/// Guess model for storing a player's guess
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guess {
    /// The word that was guessed
    pub word: String,

    /// Results for each letter
    pub results: Vec<LetterResult>,

    /// When the guess was made
    pub created_at: DateTime<Utc>,
}

/// Result for a single letter in a guess
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LetterResult {
    /// Letter is correct and in the right position
    Correct,

    /// Letter is in the word but in the wrong position
    WrongPosition,

    /// Letter is not in the word
    Wrong,
}

impl Game {
    /// Create a new game
    pub fn new(word: String, max_attempts: u8, user_id: Uuid) -> Self {
        let now = Utc::now();

        Self {
            id: Uuid::new_v4(),
            user_id,
            word,
            max_attempts,
            guesses: Vec::new(),
            completed: false,
            won: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if the game is completed
    pub fn is_completed(&self) -> bool {
        self.completed
    }

    /// Get the number of attempts remaining
    pub fn attempts_remaining(&self) -> u8 {
        self.max_attempts.saturating_sub(self.guesses.len() as u8)
    }

    /// Add a guess to the game
    pub fn add_guess(&mut self, guess: Guess) -> Result<(), GameError> {
        if self.attempts_remaining() == 0 {
            return Err(GameError::GameCompleted);
        }

        self.guesses.push(guess);
        self.updated_at = Utc::now();
        Ok(())
    }
}
