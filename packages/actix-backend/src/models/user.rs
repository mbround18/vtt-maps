use chrono::{DateTime, Utc};
use diesel::{AsChangeset, Identifiable, Insertable, Queryable, Selectable};
use shared::oauth::DiscordUserInfo;
use uuid::Uuid;

use crate::auth::models::{AuthenticatedUser, UserRole};
use crate::schema::users;

#[derive(Debug, Clone, Queryable, Identifiable, Selectable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: Uuid,
    pub discord_id: String,
    pub username: String,
    pub avatar_url: Option<String>,
    pub email: Option<String>,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn normalized_role(&self) -> UserRole {
        match self.role.as_str() {
            "admin" => UserRole::Admin,
            "contributor" => UserRole::Contributor,
            _ => UserRole::Guest,
        }
    }
}

impl From<User> for AuthenticatedUser {
    fn from(value: User) -> Self {
        let role = value.normalized_role();
        let User {
            id,
            discord_id,
            username,
            avatar_url,
            ..
        } = value;
        AuthenticatedUser {
            id,
            discord_id,
            username,
            avatar_url,
            role,
        }
    }
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub id: Uuid,
    pub discord_id: String,
    pub username: String,
    pub avatar_url: Option<String>,
    pub email: Option<String>,
    pub role: String,
}

impl NewUser {
    pub fn new_from_discord(discord: &DiscordUserInfo, role: UserRole) -> Self {
        Self {
            id: Uuid::new_v4(),
            discord_id: discord.id.clone(),
            username: preferred_name(discord),
            avatar_url: discord.avatar_url(),
            email: discord.email.clone(),
            role: format_role(role),
        }
    }
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = users)]
pub struct UpdateUser {
    pub username: Option<String>,
    pub avatar_url: Option<Option<String>>,
    pub email: Option<Option<String>>,
}

impl UpdateUser {
    pub fn new_from_discord(discord: &DiscordUserInfo) -> Self {
        Self {
            username: Some(preferred_name(discord)),
            avatar_url: Some(discord.avatar_url()),
            email: Some(discord.email.clone()),
        }
    }
}

fn format_role(role: UserRole) -> String {
    match role {
        UserRole::Admin => "admin".to_string(),
        UserRole::Contributor => "contributor".to_string(),
        UserRole::Guest => "guest".to_string(),
    }
}

fn preferred_name(discord: &DiscordUserInfo) -> String {
    discord
        .global_name
        .clone()
        .unwrap_or_else(|| discord.username.clone())
}
