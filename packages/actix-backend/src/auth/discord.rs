use reqwest::Client;
use serde::Serialize;
use thiserror::Error;

use shared::oauth::{DiscordUserInfo, OAuthProvider, OAuthProviderError, OAuthTokenResponse};

const USER_AGENT: &str = "vtt-maps-actix-backend/0.1";

/// Handles Discord specific OAuth interactions (authorize URL, token exchange, user info).
#[derive(Clone)]
pub struct DiscordOAuthClient {
    pub provider: OAuthProvider,
    http: Client,
}

impl DiscordOAuthClient {
    pub fn from_env() -> Result<Self, DiscordOAuthError> {
        let provider = OAuthProvider::discord_from_env()?;
        Self::new(provider)
    }

    pub fn new(provider: OAuthProvider) -> Result<Self, DiscordOAuthError> {
        let http = Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .map_err(DiscordOAuthError::Http)?;
        Ok(Self { provider, http })
    }

    pub fn authorize_url(&self, state: &str) -> Result<String, DiscordOAuthError> {
        Ok(self.provider.build_authorize_url(state)?.into())
    }

    pub async fn exchange_code(&self, code: &str) -> Result<OAuthTokenResponse, DiscordOAuthError> {
        let payload = DiscordTokenRequest {
            client_id: &self.provider.client_id,
            client_secret: &self.provider.client_secret,
            grant_type: "authorization_code",
            code,
            redirect_uri: &self.provider.redirect_uri,
        };

        self.http
            .post(&self.provider.token_url)
            .form(&payload)
            .send()
            .await
            .map_err(DiscordOAuthError::Http)?
            .error_for_status()
            .map_err(DiscordOAuthError::Http)?
            .json::<OAuthTokenResponse>()
            .await
            .map_err(DiscordOAuthError::Http)
    }

    pub async fn fetch_user(
        &self,
        access_token: &str,
    ) -> Result<DiscordUserInfo, DiscordOAuthError> {
        let url = self
            .provider
            .userinfo_url
            .as_ref()
            .ok_or(DiscordOAuthError::MissingUserInfoUrl)?;

        self.http
            .get(url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(DiscordOAuthError::Http)?
            .error_for_status()
            .map_err(DiscordOAuthError::Http)?
            .json::<DiscordUserInfo>()
            .await
            .map_err(DiscordOAuthError::Http)
    }
}

#[derive(Serialize)]
struct DiscordTokenRequest<'a> {
    client_id: &'a str,
    client_secret: &'a str,
    grant_type: &'a str,
    code: &'a str,
    redirect_uri: &'a str,
}

#[derive(Debug, Error)]
pub enum DiscordOAuthError {
    #[error(transparent)]
    Provider(#[from] OAuthProviderError),
    #[error("missing Discord user info endpoint")]
    MissingUserInfoUrl,
    #[error(transparent)]
    Http(#[from] reqwest::Error),
}
