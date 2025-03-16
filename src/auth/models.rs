use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User model for storing game-related information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique user identifier
    pub id: Uuid,

    /// User's username
    pub username: String,

    /// When the user was created
    pub created_at: DateTime<Utc>,

    /// When the user was last updated
    pub updated_at: DateTime<Utc>,

    /// ID of today's game for this user (if exists)
    pub current_game_id: Option<Uuid>,
}

/// JWT Claims structure for token verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,

    /// Username
    pub username: String,

    /// Issued at timestamp
    pub iat: usize,

    /// Expiration timestamp
    pub exp: usize,

    /// Issuer (Optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,

    /// Audience (Optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<Vec<String>>,

    /// Roles (Optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,

    /// Email (Optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// Name (Optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// User response structure (without sensitive data)
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub current_game_id: Option<Uuid>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            created_at: user.created_at,
            current_game_id: user.current_game_id,
        }
    }
}

impl User {
    /// Create a new user record from token information
    pub fn new(user_id: Uuid, username: String) -> Self {
        let now = Utc::now();

        Self {
            id: user_id,
            username,
            created_at: now,
            updated_at: now,
            current_game_id: None,
        }
    }
}
