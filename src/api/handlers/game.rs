use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::{Auth, AuthUserId};
use crate::core::AppState;
use crate::game::error::GameError;
use crate::game::models::{Game, LetterResult as ModelLetterResult};
use crate::repository::error::RepositoryError;

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

#[derive(Debug, Serialize, Clone, Copy)]
pub enum LetterResult {
    Correct,
    WrongPosition,
    Wrong,
}

impl From<ModelLetterResult> for LetterResult {
    fn from(result: ModelLetterResult) -> Self {
        match result {
            ModelLetterResult::Correct => LetterResult::Correct,
            ModelLetterResult::WrongPosition => LetterResult::WrongPosition,
            ModelLetterResult::Wrong => LetterResult::Wrong,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateGameRequest {
    // Empty struct
}

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
                results: g.results.iter().map(|&r| r.into()).collect(),
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

/// Create a new game
#[debug_handler]
pub async fn create_game(
    State(state): State<Arc<AppState>>,
    auth: Auth,
    Json(_request): Json<CreateGameRequest>,
) -> Result<Json<GameResponse>, GameError> {
    tracing::info!(
        "Creating new game for user: {} ({})",
        auth.claims.username,
        auth.user_id
    );

    // Check if a new day started - this ensures everyone gets the same word
    tracing::info!("Checking and updating date");
    state.check_and_update_date().await?;
    tracing::info!("Date check completed successfully");

    // Check if user already has an existing game for today
    tracing::info!("Checking if user has an existing game");
    let existing_game_result = state.get_current_user_game(&auth.user_id).await;

    // Only process existing game if found, ignore NotFound errors
    match existing_game_result {
        Ok(Some(game)) => {
            tracing::info!("Found existing game: {}", game.id);
            return Ok(Json(GameResponse::from(game)));
        }
        Ok(None) => tracing::info!("No existing game found"),
        Err(err) => {
            if matches!(err, RepositoryError::NotFound) {
                tracing::info!("User not found, will create new user");
            } else {
                tracing::error!("Error checking for existing game: {}", err);
                return Err(err.into());
            }
        }
    }

    // Get the game service and select today's word
    let game_service = state.game_service();
    let word = game_service.select_daily_word();
    tracing::debug!("Selected daily word for new game");

    // Create game with the user's ID
    let game = Game::new(word, 6, auth.user_id);
    tracing::info!("New game created: {}", game.id);

    // Save the game in our state
    tracing::info!("Saving game to repository");
    state.save_game(game.clone()).await?;
    tracing::info!("Game saved successfully");

    // Create a new user record if not exists
    let user_exists = state.get_user(&auth.user_id).await.is_ok();
    if !user_exists {
        // Create a new user record using JWT claims information
        tracing::info!(
            "Creating new user record for: {} ({})",
            auth.claims.username,
            auth.user_id
        );
        let user = crate::auth::models::User::new(auth.user_id, auth.claims.username.clone());

        // Save user
        tracing::info!("Saving new user");
        state.save_user(user).await?;
        tracing::info!("User saved successfully");
    } else {
        tracing::info!("User already exists");
    }

    // Update the user's current game reference
    tracing::info!("Updating user's current game reference");
    state.update_user_game(&auth.user_id, game.id).await?;
    tracing::info!("User game reference updated successfully");

    // Return the game response
    tracing::info!("Returning game response");
    Ok(Json(GameResponse::from(game)))
}

/// Get user's current game state
#[debug_handler]
pub async fn get_game(
    State(state): State<Arc<AppState>>,
    auth_user_id: AuthUserId,
    Path(game_id): Path<Uuid>,
) -> Result<Json<GameResponse>, GameError> {
    // Check for day change
    state.check_and_update_date().await?;

    // Get game
    let game = state.get_game(&game_id).await?;

    // Verify game ownership
    if game.user_id != auth_user_id.0 {
        return Err(GameError::GameNotFound);
    }

    // Return game response
    Ok(Json(GameResponse::from(game)))
}

/// Make a guess for the current game
#[debug_handler]
pub async fn make_guess(
    State(state): State<Arc<AppState>>,
    auth_user_id: AuthUserId,
    Path(game_id): Path<Uuid>,
    Json(request): Json<GuessRequest>,
) -> Result<Json<GameResponse>, GameError> {
    // Get game
    let mut game = state.get_game(&game_id).await?;

    // Verify game ownership
    if game.user_id != auth_user_id.0 {
        return Err(GameError::GameNotFound);
    }

    // Get the game service
    let game_service = state.game_service();

    // Make the guess
    game_service.make_guess(&mut game, &request.word)?;

    // Save the updated game
    state.save_game(game.clone()).await?;

    // Return the updated game response
    Ok(Json(GameResponse::from(game)))
}
