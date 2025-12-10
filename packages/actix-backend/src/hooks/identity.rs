use actix_session::{SessionMiddleware, config::PersistentSession, storage::CookieSessionStore};
use actix_web::cookie::Key;
use actix_web::cookie::{SameSite, time::Duration};
use std::env;
use tracing::info;

/// Configure session & identity middleware using cookies (fallback)
pub fn session_middleware() -> SessionMiddleware<CookieSessionStore> {
    // Generate or load your 64-byte key for session encryption
    let secret_key = Key::generate();

    info!("🍪 Using cookie-based session store (single-instance mode)");

    SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
        .cookie_name("vtt-maps.dnd-apps.dev".to_string())
        .cookie_http_only(true)
        .cookie_secure(env::var("COOKIE_SECURE").is_ok())
        .cookie_same_site(SameSite::Strict)
        .session_lifecycle(PersistentSession::default().session_ttl(Duration::minutes(30)))
        .build()
}
