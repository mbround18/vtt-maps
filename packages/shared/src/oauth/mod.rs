use std::env;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

/// Discord OAuth endpoints.
pub const DISCORD_AUTHORIZE_URL: &str = "https://discord.com/api/oauth2/authorize";
pub const DISCORD_TOKEN_URL: &str = "https://discord.com/api/oauth2/token";
pub const DISCORD_USERINFO_URL: &str = "https://discord.com/api/users/@me";

/// Represents a generic OAuth provider configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OAuthProvider {
    pub kind: OAuthProviderKind,
    pub client_id: String,
    pub client_secret: String,
    pub authorize_url: String,
    pub token_url: String,
    pub userinfo_url: Option<String>,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
}

impl OAuthProvider {
    /// Builds an authorization URL including the provided CSRF `state` token.
    pub fn build_authorize_url(&self, state: &str) -> Result<Url, OAuthProviderError> {
        let mut url = Url::parse(&self.authorize_url)
            .map_err(|e| OAuthProviderError::InvalidAuthorizeUrl(e.to_string()))?;
        {
            let mut pairs = url.query_pairs_mut();
            pairs.append_pair("response_type", "code");
            pairs.append_pair("client_id", &self.client_id);
            pairs.append_pair("redirect_uri", &self.redirect_uri);
            pairs.append_pair("scope", &self.scopes.join(" "));
            pairs.append_pair("state", state);
        }
        Ok(url)
    }

    /// Helper to construct the default Discord OAuth provider.
    pub fn discord_from_env() -> Result<Self, OAuthProviderError> {
        let client_id = env_var("DISCORD_CLIENT_ID")?;
        let client_secret = env_var("DISCORD_CLIENT_SECRET")?;
        let redirect_uri = env_var("DISCORD_REDIRECT_URI")?;
        let scopes = env::var("DISCORD_OAUTH_SCOPE")
            .unwrap_or_else(|_| "identify email".to_string())
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        Ok(Self {
            kind: OAuthProviderKind::Discord,
            client_id,
            client_secret,
            authorize_url: DISCORD_AUTHORIZE_URL.to_string(),
            token_url: DISCORD_TOKEN_URL.to_string(),
            userinfo_url: Some(DISCORD_USERINFO_URL.to_string()),
            redirect_uri,
            scopes,
        })
    }
}

/// Enumeration of supported OAuth providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OAuthProviderKind {
    Discord,
}

impl OAuthProviderKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            OAuthProviderKind::Discord => "discord",
        }
    }
}

/// Standard OAuth token response payload.
#[derive(Debug, Clone, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

/// Discord user profile payload (subset).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiscordUserInfo {
    pub id: String,
    pub username: String,
    pub global_name: Option<String>,
    pub avatar: Option<String>,
    pub email: Option<String>,
}

impl DiscordUserInfo {
    /// Returns the rendered avatar URL if present.
    pub fn avatar_url(&self) -> Option<String> {
        self.avatar.as_ref().map(|hash| {
            format!(
                "https://cdn.discordapp.com/avatars/{}/{}.png?size=256",
                self.id, hash
            )
        })
    }
}

/// Errors that can occur while preparing OAuth providers.
#[derive(Debug, Error)]
pub enum OAuthProviderError {
    #[error("missing required environment variable `{0}`")]
    MissingEnv(&'static str),
    #[error("failed to parse authorize URL: {0}")]
    InvalidAuthorizeUrl(String),
}

fn env_var(key: &'static str) -> Result<String, OAuthProviderError> {
    env::var(key)
        .map_err(|_| OAuthProviderError::MissingEnv(key))
        .and_then(|value| {
            if value.trim().is_empty() {
                Err(OAuthProviderError::MissingEnv(key))
            } else {
                Ok(value)
            }
        })
}
