use chrono::{Datelike, NaiveDate, Utc};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use rand::rngs::StdRng;
use rand::{prelude::*, SeedableRng};
use std::collections::HashMap;

pub mod error;
pub mod models;
mod words;

use self::error::GameError;
use self::models::{Game, Guess, LetterResult};

// Daily word cache with more efficient Mutex implementation
static DAILY_WORD_CACHE: Lazy<Mutex<HashMap<NaiveDate, String>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Service for handling game logic
#[derive(Debug, Clone)]
pub struct GameService {
    // Dictionary of valid words
    word_list: Vec<String>,
    // Length of words used in the game
    word_length: usize,
}

impl GameService {
    /// Create a new game service
    pub fn new() -> Self {
        Self {
            word_list: words::WORD_LIST.iter().map(|&w| String::from(w)).collect(),
            word_length: 5, // Standard Wordle uses 5-letter words
        }
    }

    /// Get today's word for external use
    pub fn select_daily_word(&self) -> String {
        self.get_daily_word()
    }

    /// Make a guess in a game
    pub fn make_guess(&self, game: &mut Game, guess_word: &str) -> Result<(), GameError> {
        // Check if the game is already completed
        if game.is_completed() {
            return Err(GameError::GameCompleted);
        }

        // Convert to lowercase
        let guess_word_lower = guess_word.to_lowercase();

        // Check if the word has the correct length
        if guess_word_lower.chars().count() != self.word_length {
            return Err(GameError::InvalidWord(format!(
                "Word must be {} letters",
                self.word_length
            )));
        }

        // Check if the word is valid
        if !self.is_valid_word(&guess_word_lower) {
            return Err(GameError::InvalidWord(format!(
                "Not in word list: {}",
                guess_word_lower
            )));
        }

        // Evaluate the guess
        let results = self.evaluate_guess(&game.word, &guess_word_lower);

        // Create the guess
        let guess = Guess {
            word: guess_word_lower.clone(),
            results,
            created_at: Utc::now(),
        };

        // Update the game
        game.guesses.push(guess);
        game.updated_at = Utc::now();

        // Check if the player won
        if guess_word_lower == game.word {
            game.won = true;
            game.completed = true;
        } else if game.attempts_remaining() == 0 {
            // No more attempts left, game over
            game.completed = true;
        }

        Ok(())
    }

    /// Get today's word. All users get the same word on the same date.
    fn get_daily_word(&self) -> String {
        let today = Utc::now().date_naive();

        // Using parking_lot Mutex which doesn't require unwrap
        let mut cache = DAILY_WORD_CACHE.lock();

        // Return cached word if available, otherwise generate and cache
        cache
            .entry(today)
            .or_insert_with(|| self.generate_word_from_date(today))
            .clone()
    }

    /// Generate a word using the date as a seed
    fn generate_word_from_date(&self, date: NaiveDate) -> String {
        // Create a seed from the date (combining year, month, day)
        let seed = (date.year() as u64 * 10000) + (date.month() as u64 * 100) + date.day() as u64;

        // Initialize random number generator from the seed
        let mut rng = StdRng::seed_from_u64(seed);

        // Randomly select a word from the word list
        self.word_list
            .choose(&mut rng)
            .unwrap_or(&"hello".to_string())
            .clone()
    }

    /// Check if a word is valid
    fn is_valid_word(&self, word: &str) -> bool {
        self.word_list.contains(&word.to_string())
    }

    /// Evaluate a guess against the target word
    /// Returns a vector of LetterResult indicating the status of each letter
    fn evaluate_guess(&self, target: &str, guess: &str) -> Vec<LetterResult> {
        let target_chars: Vec<char> = target.chars().collect();
        let guess_chars: Vec<char> = guess.chars().collect();

        // Initialize results with all Wrong
        let mut results = vec![LetterResult::Wrong; guess_chars.len()];

        // First pass: Mark correct letters (exact position matches)
        for (i, (&guess_char, &target_char)) in
            guess_chars.iter().zip(target_chars.iter()).enumerate()
        {
            if guess_char == target_char {
                results[i] = LetterResult::Correct;
            }
        }

        // Count remaining characters in target (excluding correct matches)
        let mut remaining_counts = HashMap::new();
        for (i, &c) in target_chars.iter().enumerate() {
            if i >= results.len() || results[i] != LetterResult::Correct {
                *remaining_counts.entry(c).or_insert(0) += 1;
            }
        }

        // Second pass: Mark wrong position matches
        for (i, &c) in guess_chars.iter().enumerate() {
            if results[i] != LetterResult::Correct {
                if let Some(count) = remaining_counts.get_mut(&c) {
                    if *count > 0 {
                        results[i] = LetterResult::WrongPosition;
                        *count -= 1;
                    }
                }
            }
        }

        results
    }
}

// Implement Default for GameService
impl Default for GameService {
    fn default() -> Self {
        Self::new()
    }
}
