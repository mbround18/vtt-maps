use actix_session::SessionMiddleware;
use actix_session::storage::RedisSessionStore;
use actix_web::cookie::Key;
use actix_web::cookie::{SameSite, time::Duration};
use anyhow::Result;
use base64::Engine;
use std::env;
use tracing::{info, warn};

/// Attempts to create a Redis-backed session store  
/// Falls back to Cookie-backed session store if Redis is unavailable
///
/// NOTE: This is called during server startup. If Redis is unavailable,
/// the application falls back to cookie-based sessions automatically.
/// See ADR-0002 for context: docs/adrs/0002-redis-session-store.md
pub async fn create_redis_session_middleware() -> Result<SessionMiddleware<RedisSessionStore>> {
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    match RedisSessionStore::new(&redis_url).await {
        Ok(store) => {
            info!("✅ Redis session store initialized at {}", redis_url);

            let secret_key = load_or_generate_session_key();

            Ok(SessionMiddleware::builder(store, secret_key)
                .cookie_name("vtt-maps.dnd-apps.dev".to_string())
                .cookie_http_only(true)
                .cookie_secure(env::var("COOKIE_SECURE").is_ok())
                .cookie_same_site(SameSite::Strict)
                .session_lifecycle(
                    actix_session::config::PersistentSession::default()
                        .session_ttl(Duration::minutes(30)),
                )
                .build())
        }
        Err(e) => {
            warn!(
                "⚠️  Redis session store failed ({}), falling back to cookie-based sessions. \
                 For multi-instance deployment, ensure REDIS_URL is set to a running Redis server.",
                e
            );
            Err(e)
        }
    }
}

/// Load session key from environment or generate a new one
/// In production, set SESSION_KEY to persist the key across restarts
fn load_or_generate_session_key() -> Key {
    if let Ok(key_base64) = env::var("SESSION_KEY") {
        if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(&key_base64)
            && bytes.len() == 64
        {
            return Key::from(bytes.as_slice());
        }
        warn!("SESSION_KEY is invalid (must be base64-encoded 64-byte key), generating new one");
    }
    Key::generate()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_or_generate_session_key_no_env() {
        // When SESSION_KEY is not set, should generate a new key
        // SAFETY: Safe in test context, we're just cleaning up
        unsafe {
            env::remove_var("SESSION_KEY");
        }
        let key = load_or_generate_session_key();
        assert!(!key.master().is_empty());
    }

    #[test]
    fn test_load_or_generate_session_key_from_env() {
        // Generate a valid 64-byte key in base64
        let key_bytes = vec![0u8; 64];
        let key_base64 = base64::engine::general_purpose::STANDARD.encode(&key_bytes);
        // SAFETY: Safe in test context, we're testing SESSION_KEY loading
        unsafe {
            env::set_var("SESSION_KEY", &key_base64);
        }

        let key = load_or_generate_session_key();
        assert!(!key.master().is_empty());

        // SAFETY: Safe in test context, we're just cleaning up
        unsafe {
            env::remove_var("SESSION_KEY");
        }
    }

    #[test]
    fn test_load_or_generate_session_key_invalid_env() {
        // When SESSION_KEY is invalid, should generate a new key
        // SAFETY: Safe in test context, we're testing error handling
        unsafe {
            env::set_var("SESSION_KEY", "invalid-base64!!!");
        }
        let key = load_or_generate_session_key();
        assert!(!key.master().is_empty());

        // SAFETY: Safe in test context, we're just cleaning up
        unsafe {
            env::remove_var("SESSION_KEY");
        }
    }
}
