use axum::{
    Json,
    extract::{Path, State},
};
use axum_macros::debug_handler;
use uuid::Uuid;

use crate::api::AppState;
use crate::api::models::{CreateGameRequest, GameResponse, GuessRequest};
use crate::auth::{Auth, AuthUserId};
use crate::game::error::GameError;
use crate::game::models::Game;

/// Create a new game
#[debug_handler]
pub async fn create_game(
    State(state): State<AppState>,
    auth: Auth,
    Json(_request): Json<CreateGameRequest>,
) -> Result<Json<GameResponse>, GameError> {
    tracing::info!(
        "Creating new game for user: {} ({})",
        auth.claims.username,
        auth.user_id
    );

    // Check if a new day started
    tracing::info!("Checking and updating date");
    state.game.check_and_update_date().await?;

    // Check if user already has an existing game for today
    tracing::info!("Checking if user has an existing game");
    let existing_game_id = state.auth.get_current_user_game_id(&auth.user_id).await?;

    // Only process existing game if found
    if let Some(game_id) = existing_game_id {
        tracing::info!("Found existing game: {}", game_id);
        let game = state.game.get_game(&game_id).await?;
        return Ok(Json(GameResponse::from(game)));
    }

    tracing::info!("No existing game found");

    // Get the game service and select today's word
    let game_service = state.game.game_service();
    let word = game_service.select_daily_word();
    tracing::debug!("Selected daily word for new game");

    // Create game with the user's ID
    let game = Game::new(word, 6, auth.user_id);
    tracing::info!("New game created: {}", game.id);

    // Save the game in our state
    tracing::info!("Saving game to repository");
    state.game.save_game(game.clone()).await?;
    tracing::info!("Game saved successfully");

    // Create a new user record if not exists
    let user_exists = state.auth.get_user(&auth.user_id).await.is_ok();
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
        state.auth.save_user(user).await?;
    } else {
        tracing::info!("User already exists");
    }

    // Update the user's current game reference
    tracing::info!("Updating user's current game reference");
    state.auth.update_user_game(&auth.user_id, game.id).await?;

    // Return the game response
    tracing::info!("Returning game response");
    Ok(Json(GameResponse::from(game)))
}

/// Get user's current game state
#[debug_handler]
pub async fn get_game(
    State(state): State<AppState>,
    auth_user_id: AuthUserId,
    Path(game_id): Path<Uuid>,
) -> Result<Json<GameResponse>, GameError> {
    // Check for day change
    state.game.check_and_update_date().await?;

    // Get game
    let game = state.game.get_game(&game_id).await?;

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
    State(state): State<AppState>,
    auth_user_id: AuthUserId,
    Path(game_id): Path<Uuid>,
    Json(request): Json<GuessRequest>,
) -> Result<Json<GameResponse>, GameError> {
    // Get game
    let mut game = state.game.get_game(&game_id).await?;

    // Verify game ownership
    if game.user_id != auth_user_id.0 {
        return Err(GameError::GameNotFound);
    }

    // Get the game service
    let game_service = state.game.game_service();

    // Make the guess
    game_service.make_guess(&mut game, &request.word)?;

    // Save the updated game
    state.game.save_game(game.clone()).await?;

    // Return the updated game response
    Ok(Json(GameResponse::from(game)))
}
