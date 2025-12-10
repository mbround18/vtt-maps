use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Supported user roles propagated through JWT claims.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Contributor,
    #[default]
    Guest,
}

impl UserRole {
    pub fn is_admin(self) -> bool {
        matches!(self, UserRole::Admin)
    }

    pub fn is_contributor(self) -> bool {
        matches!(self, UserRole::Admin | UserRole::Contributor)
    }
}

/// Normalized representation of an authenticated Discord user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub discord_id: String,
    pub username: String,
    pub avatar_url: Option<String>,
    pub role: UserRole,
}

impl AuthenticatedUser {
    pub fn is_admin(&self) -> bool {
        self.role.is_admin()
    }
}
