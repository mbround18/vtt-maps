use std::env;

use anyhow::{Context, anyhow};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use tracing::{info, warn};

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

const DEFAULT_USER: &str = "postgres";
const DEFAULT_PASSWORD: &str = "postgres";
const DEFAULT_DB_NAME: &str = "vttmaps";
const DEFAULT_PORT: &str = "5432";
const CLUSTER_HOST: &str = "postgres.vtt-maps.svc.cluster.local";
const LOCAL_HOST: &str = "localhost";

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn init_pool() -> anyhow::Result<DbPool> {
    let database_url = database_url();
    info!("📦 Using database URL: {}", redact_password(&database_url));

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder()
        .build(manager)
        .context("Failed to build Postgres pool")?;

    // Run embedded migrations on startup
    {
        let mut conn = pool
            .get()
            .context("Failed to acquire DB connection for migrations")?;
        info!("📚 Running database migrations (if any pending)");
        conn.run_pending_migrations(MIGRATIONS)
            .map_err(|e| anyhow!("Failed to run database migrations: {e}"))?;
    }

    Ok(pool)
}

fn database_url() -> String {
    if let Ok(url) = env::var("DATABASE_URL")
        && !url.trim().is_empty()
    {
        return url;
    }

    let user = env::var("DATABASE_USER").unwrap_or_else(|_| DEFAULT_USER.to_string());
    let password = env::var("DATABASE_PASSWORD").unwrap_or_else(|_| DEFAULT_PASSWORD.to_string());
    let db_name = env::var("DATABASE_NAME").unwrap_or_else(|_| DEFAULT_DB_NAME.to_string());
    let port = env::var("DATABASE_PORT").unwrap_or_else(|_| DEFAULT_PORT.to_string());

    let host = env::var("DATABASE_HOST").unwrap_or_else(|_| {
        if env::var("CONTAINER").is_ok() {
            CLUSTER_HOST.to_string()
        } else {
            LOCAL_HOST.to_string()
        }
    });

    warn!(
        "DATABASE_URL not provided, falling back to postgres://<redacted>@{}:{}/{}",
        host, port, db_name
    );
    format!("postgres://{user}:{password}@{host}:{port}/{db_name}")
}

fn redact_password(url: &str) -> String {
    if let Some((prefix, rest)) = url.split_once("//")
        && let Some((creds, suffix)) = rest.split_once('@')
        && creds.contains(':')
    {
        let user = creds.split(':').next().unwrap_or("");
        return format!("{prefix}//{user}:***@{suffix}");
    }
    url.to_string()
}
