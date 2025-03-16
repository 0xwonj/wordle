// Re-export API response models from their original locations
pub use crate::api::handlers::game::{GameResponse, GuessResponse, LetterResult};

// Re-export API request models
pub use crate::api::handlers::game::{CreateGameRequest, GuessRequest};
