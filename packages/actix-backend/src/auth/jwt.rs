use std::env;

use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use actix_web::cookie::{Cookie, SameSite, time::Duration as CookieDuration};
use uuid::Uuid;

use crate::auth::models::{AuthenticatedUser, UserRole};

const DEFAULT_TTL_MINUTES: i64 = 60;
pub const SESSION_COOKIE_NAME: &str = "vttmaps.session";

/// JSON Web Token service responsible for signing and validating user sessions.
#[derive(Clone)]
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    issuer: String,
    ttl: Duration,
}

impl JwtService {
    /// Build a service instance from environment variables.
    pub fn from_env() -> Result<Self, JwtError> {
        let secret = env_var("JWT_SECRET")?;
        let issuer = env::var("JWT_ISSUER").unwrap_or_else(|_| "vtt-maps".to_string());
        let ttl_minutes = env::var("JWT_TTL_MINUTES")
            .ok()
            .and_then(|raw| raw.parse::<i64>().ok())
            .unwrap_or(DEFAULT_TTL_MINUTES);

        Ok(Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            issuer,
            ttl: Duration::minutes(ttl_minutes.max(1)),
        })
    }

    /// Generates a signed JWT for the provided user payload.
    pub fn sign(&self, user: &AuthenticatedUser) -> Result<String, JwtError> {
        let now = Utc::now();
        let exp = now + self.ttl;
        let claims = JwtClaims {
            sub: user.id.to_string(),
            iss: self.issuer.clone(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
            discord_id: user.discord_id.clone(),
            username: user.username.clone(),
            avatar_url: user.avatar_url.clone(),
            role: user.role,
        };
        encode(&Header::new(Algorithm::HS256), &claims, &self.encoding_key).map_err(JwtError::Token)
    }

    /// Verifies the supplied JWT and converts it back into an `AuthenticatedUser`.
    pub fn verify(&self, token: &str) -> Result<AuthenticatedUser, JwtError> {
        let data = decode::<JwtClaims>(token, &self.decoding_key, &self.validation())
            .map_err(JwtError::Token)?;
        Ok(data.claims.into())
    }

    /// Builds a secure, HTTP-only session cookie for the provided token.
    pub fn session_cookie(&self, token: &str) -> Cookie<'static> {
        Cookie::build(SESSION_COOKIE_NAME, token.to_string())
            .http_only(true)
            .same_site(SameSite::Strict)
            .secure(cookie_secure())
            .path("/")
            .max_age(CookieDuration::seconds(self.ttl.num_seconds()))
            .finish()
    }

    /// Returns a cookie instructing the browser to clear the session token.
    pub fn clear_session_cookie(&self) -> Cookie<'static> {
        Cookie::build(SESSION_COOKIE_NAME, "")
            .http_only(true)
            .same_site(SameSite::Strict)
            .secure(cookie_secure())
            .path("/")
            .max_age(CookieDuration::seconds(0))
            .finish()
    }

    fn validation(&self) -> Validation {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(std::slice::from_ref(&self.issuer));
        validation
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct JwtClaims {
    sub: String,
    iss: String,
    iat: i64,
    exp: i64,
    discord_id: String,
    username: String,
    avatar_url: Option<String>,
    role: UserRole,
}

impl From<JwtClaims> for AuthenticatedUser {
    fn from(value: JwtClaims) -> Self {
        let id = Uuid::parse_str(&value.sub).unwrap_or_else(|_| Uuid::nil());
        AuthenticatedUser {
            id,
            discord_id: value.discord_id,
            username: value.username,
            avatar_url: value.avatar_url,
            role: value.role,
        }
    }
}

#[derive(Debug, Error)]
pub enum JwtError {
    #[error("missing required environment variable `{0}`")]
    MissingEnv(&'static str),
    #[error("failed to sign or verify token: {0}")]
    Token(#[from] jsonwebtoken::errors::Error),
}

fn env_var(key: &'static str) -> Result<String, JwtError> {
    env::var(key)
        .map_err(|_| JwtError::MissingEnv(key))
        .and_then(|value| {
            if value.trim().is_empty() {
                Err(JwtError::MissingEnv(key))
            } else {
                Ok(value)
            }
        })
}

fn cookie_secure() -> bool {
    env::var("COOKIE_SECURE").is_ok()
}
