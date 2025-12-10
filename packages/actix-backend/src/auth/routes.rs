use std::env;

use actix_web::cookie::{Cookie, SameSite, time::Duration as CookieDuration};
use actix_web::{HttpRequest, HttpResponse, Result, error, http::header, web};
use anyhow::Context;
use diesel::prelude::*;
use serde::Deserialize;
use tracing::error;
use uuid::Uuid;

use crate::auth::discord::DiscordOAuthClient;
use crate::auth::extractor::AuthenticatedSession;
use crate::auth::jwt::JwtService;
use crate::auth::models::{AuthenticatedUser, UserRole};
use crate::models::user::{NewUser, UpdateUser, User};
use crate::schema::users::dsl::{discord_id as discord_id_col, users as users_table};
use crate::utils::db::DbPool;
use shared::oauth::DiscordUserInfo;

const STATE_COOKIE_NAME: &str = "vttmaps.oauth_state";
const STATE_COOKIE_TTL_MINUTES: i64 = 10;

#[derive(Debug, Deserialize)]
pub struct DiscordCallbackQuery {
    code: String,
    state: String,
}

pub async fn discord_start(client: web::Data<DiscordOAuthClient>) -> Result<HttpResponse> {
    let state = Uuid::new_v4().to_string();
    let authorize_url = client.authorize_url(&state).map_err(|e| {
        error!("Failed to build Discord authorize URL: {e:?}");
        error::ErrorInternalServerError("failed to start OAuth flow")
    })?;

    Ok(HttpResponse::Found()
        .append_header((header::LOCATION, authorize_url))
        .cookie(build_state_cookie(&state))
        .finish())
}

pub async fn discord_callback(
    query: web::Query<DiscordCallbackQuery>,
    req: HttpRequest,
    client: web::Data<DiscordOAuthClient>,
    jwt: web::Data<JwtService>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse> {
    let state_valid = req
        .cookie(STATE_COOKIE_NAME)
        .map(|cookie| cookie.value() == query.state)
        .unwrap_or(false);

    if !state_valid {
        return Err(error::ErrorUnauthorized("invalid OAuth state"));
    }

    let tokens = client.exchange_code(&query.code).await.map_err(|e| {
        error!("Discord token exchange failed: {e:?}");
        error::ErrorInternalServerError("failed to complete OAuth flow")
    })?;

    let profile = client.fetch_user(&tokens.access_token).await.map_err(|e| {
        error!("Discord user fetch failed: {e:?}");
        error::ErrorInternalServerError("failed to fetch Discord user")
    })?;

    let insert_role = resolve_insert_role(&profile.id);
    let pool_clone = pool.clone();
    let profile_clone = profile.clone();

    let user = web::block(move || upsert_user(&pool_clone, &profile_clone, insert_role))
        .await
        .map_err(|e| {
            error!("User upsert blocking task failed: {e:?}");
            error::ErrorInternalServerError("failed to persist user")
        })?
        .map_err(|e| {
            error!("User upsert failed: {e:?}");
            error::ErrorInternalServerError("failed to persist user")
        })?;

    let auth_user: AuthenticatedUser = user.into();
    let token = jwt.sign(&auth_user).map_err(|e| {
        error!("Failed to sign JWT: {e:?}");
        error::ErrorInternalServerError("failed to create session")
    })?;

    Ok(HttpResponse::Found()
        .append_header((header::LOCATION, success_redirect()))
        .cookie(jwt.session_cookie(&token))
        .cookie(clear_state_cookie())
        .finish())
}

pub async fn current_user(session: AuthenticatedSession) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(session.user()))
}

pub async fn logout(jwt: web::Data<JwtService>) -> Result<HttpResponse> {
    Ok(HttpResponse::NoContent()
        .cookie(jwt.clear_session_cookie())
        .cookie(clear_state_cookie())
        .finish())
}

fn upsert_user(
    pool: &DbPool,
    profile: &DiscordUserInfo,
    role: UserRole,
) -> Result<User, anyhow::Error> {
    let mut conn = pool.get().context("failed to acquire DB connection")?;
    let user = diesel::insert_into(users_table)
        .values(&NewUser::new_from_discord(profile, role))
        .on_conflict(discord_id_col)
        .do_update()
        .set(&UpdateUser::new_from_discord(profile))
        .returning(User::as_returning())
        .get_result::<User>(&mut conn)
        .context("failed to upsert user row")?;
    Ok(user)
}

fn resolve_insert_role(discord_id: &str) -> UserRole {
    if id_in_env("DISCORD_ADMIN_IDS", discord_id) {
        UserRole::Admin
    } else if id_in_env("DISCORD_CONTRIBUTOR_IDS", discord_id) {
        UserRole::Contributor
    } else {
        UserRole::Guest
    }
}

fn id_in_env(key: &str, needle: &str) -> bool {
    env::var(key)
        .ok()
        .map(|csv| csv.split(',').any(|id| id.trim() == needle))
        .unwrap_or(false)
}

fn build_state_cookie(value: &str) -> Cookie<'static> {
    Cookie::build(STATE_COOKIE_NAME, value.to_string())
        .http_only(true)
        .same_site(SameSite::Strict)
        .secure(cookie_secure())
        .path("/")
        .max_age(CookieDuration::minutes(STATE_COOKIE_TTL_MINUTES))
        .finish()
}

fn clear_state_cookie() -> Cookie<'static> {
    Cookie::build(STATE_COOKIE_NAME, "")
        .http_only(true)
        .same_site(SameSite::Strict)
        .secure(cookie_secure())
        .path("/")
        .max_age(CookieDuration::seconds(0))
        .finish()
}

fn cookie_secure() -> bool {
    env::var("COOKIE_SECURE").is_ok()
}

fn success_redirect() -> String {
    env::var("OAUTH_SUCCESS_REDIRECT").unwrap_or_else(|_| "/".to_string())
}
