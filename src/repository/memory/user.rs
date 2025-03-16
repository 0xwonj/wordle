use async_trait::async_trait;
use chrono::Utc;
use parking_lot::RwLock;
use std::collections::HashMap;
use uuid::Uuid;

use crate::auth::models::User;
use crate::repository::UserRepositoryTrait;
use crate::repository::error::{RepositoryError, RepositoryResult};

/// In-memory implementation of user repository
/// Useful for testing and development environments
#[derive(Debug, Default)]
pub struct InMemoryUserRepository {
    /// In-memory user storage, keyed by user ID
    users: RwLock<HashMap<Uuid, User>>,
}

impl InMemoryUserRepository {
    /// Create a new in-memory user repository
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl UserRepositoryTrait for InMemoryUserRepository {
    /// Get a user by ID
    async fn get_user(&self, id: &Uuid) -> RepositoryResult<User> {
        let users = self.users.read();
        users.get(id).cloned().ok_or(RepositoryError::NotFound)
    }

    /// Save a user
    async fn save_user(&self, user: User) -> RepositoryResult<()> {
        let mut users = self.users.write();
        users.insert(user.id, user);
        Ok(())
    }

    /// Update a user's current game ID
    async fn update_user_game(&self, user_id: &Uuid, game_id: Uuid) -> RepositoryResult<bool> {
        let mut users = self.users.write();

        if let Some(user) = users.get_mut(user_id) {
            user.current_game_id = Some(game_id);
            user.updated_at = Utc::now();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Reset all users' current game IDs and return the count of updated users
    async fn reset_all_users_current_game(&self) -> RepositoryResult<usize> {
        let mut users = self.users.write();
        let updated_count = users.len();

        // Clear the current game ID for all users
        for user in users.values_mut() {
            user.current_game_id = None;
            user.updated_at = Utc::now();
        }

        Ok(updated_count)
    }
}
