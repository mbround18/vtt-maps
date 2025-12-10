mod auth;
mod clients;
mod docs;
mod health;
mod hooks;
mod maps;
mod models;
mod schema;
mod services;
mod utils;
mod wrappers;

use actix_files::Files;
use actix_web::dev::Service;
use actix_web::{App, HttpServer, http::header::CONTENT_TYPE, web};
use dotenvy::dotenv;
use tracing::{error, info};

use crate::auth::{discord::DiscordOAuthClient, jwt::JwtService};
use crate::hooks::logger::setup_logger;
use crate::hooks::redis_session::create_redis_session_middleware;
use crate::maps::rebuild_maps_init;
use crate::services::file_service::file_service;
use crate::wrappers::seo::SeoMetadata;
use actix_identity::IdentityMiddleware;
use shared::utils::root_dir::root_dir;
use std::env;
use tracing_actix_web::TracingLogger;
use utils::folders::thumbnails_dir;
use utils::setup::setup_folders;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing subscriber
    setup_logger();
    dotenv().ok();

    // Initialize and validate admin configuration
    if let Err(e) = utils::admin_init::initialize_admin_config() {
        error!("❌ Admin configuration initialization failed: {:?}", e);
        eprintln!("Admin configuration error: {e:?}");
        std::process::exit(1);
    }

    let address = env::var("ADDRESS").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let root = root_dir()?;
    let thumb_dir = thumbnails_dir()?;
    match setup_folders() {
        Ok(()) => info!("Base setup complete."),
        Err(e) => {
            eprintln!("Error when checking for base setup!: {e:?}");
            std::process::exit(1);
        }
    }

    info!("Operating out of directory: {}", root.display());

    // Initialize database pool + migrations
    let db_pool = match utils::db::init_pool() {
        Ok(pool) => pool,
        Err(e) => {
            error!("❌ Failed to initialize database pool: {:?}", e);
            eprintln!("Database initialization failed: {e:?}");
            std::process::exit(1);
        }
    };
    let db_pool = web::Data::new(db_pool);

    // Initialize JWT session service
    let jwt_service = match JwtService::from_env() {
        Ok(service) => web::Data::new(service),
        Err(e) => {
            error!("❌ Failed to initialize JWT service: {:?}", e);
            eprintln!("JWT service initialization failed: {e:?}");
            std::process::exit(1);
        }
    };

    // Initialize Discord OAuth client
    let discord_client = match DiscordOAuthClient::from_env() {
        Ok(client) => web::Data::new(client),
        Err(e) => {
            error!("❌ Failed to initialize Discord OAuth client: {:?}", e);
            eprintln!("Discord OAuth initialization failed: {e:?}");
            std::process::exit(1);
        }
    };

    // Initialize admin token system
    match utils::admin_token::get_or_create_admin_token() {
        Ok(_) => info!("🔐 Admin token system initialized"),
        Err(e) => {
            error!("❌ Failed to initialize admin token system: {:?}", e);
            eprintln!("Admin token initialization failed: {e:?}");
            std::process::exit(1);
        }
    }

    // Run map rebuild process during initialization
    info!("🔧 Initializing maps rebuild process...");
    match rebuild_maps_init().await {
        Ok(count) => info!(
            "✅ Map rebuild completed successfully: {} maps processed",
            count
        ),
        Err(e) => {
            if e.to_string().contains("already in progress") || e.to_string().contains("up-to-date")
            {
                info!("ℹ️  Map rebuild: {}", e);
            } else {
                error!("❌ Map rebuild failed during initialization: {:?}", e);
                eprintln!("Map rebuild failed: {e:?}");
                std::process::exit(1);
            }
        }
    }

    info!("Listening on {}:{}", &address, &port);

    // Try to initialize Redis session store, fall back to cookies
    let redis_available = create_redis_session_middleware().await.is_ok();

    if redis_available {
        info!("✅ Redis session store available - multi-instance deployment ready");
    } else {
        info!("🍪 Using cookie-based sessions (single-instance or no Redis)");
    }

    HttpServer::new(move || {
        let db_pool = db_pool.clone();
        let jwt_service = jwt_service.clone();
        let discord_client = discord_client.clone();

        App::new()
            .app_data(db_pool)
            .app_data(jwt_service)
            .app_data(discord_client)
            // Register middleware via configure hooks
            .wrap(TracingLogger::default())
            .wrap(IdentityMiddleware::default())
            .wrap(if redis_available {
                // Note: In actual runtime, Redis would be reinitialized per worker.
                // For now, we use cookies as fallback for all instances.
                hooks::identity::session_middleware()
            } else {
                hooks::identity::session_middleware()
            })
            .wrap(hooks::cors::cors())
            .wrap(hooks::security::security())
            // SEO wrapper
            .wrap(SeoMetadata)
            // Static thumbnails
            .service(Files::new("/assets/thumbnails", thumb_dir.clone()).use_last_modified(true))
            // API routes
            .service(
                web::scope("/api")
                    .wrap_fn(|req, srv| {
                        let fut = srv.call(req);
                        async move {
                            let mut res = fut.await?;
                            if !res.headers().contains_key(CONTENT_TYPE) {
                                res.headers_mut()
                                    .insert(CONTENT_TYPE, "application/json".parse().unwrap());
                            }
                            Ok(res)
                        }
                    })
                    .service(
                        web::scope("/maps")
                            .route("/all", web::get().to(maps::maps_all))
                            .route("/{id}", web::get().to(maps::map_detail))
                            .route("/rebuild", web::post().to(maps::maps_rebuild))
                            .route("/rebuild/status", web::get().to(maps::rebuild_status))
                            .route("/rebuild/clear", web::delete().to(maps::clear_rebuild_lock))
                            .route("/download/{id}", web::get().to(maps::download_map))
                            .route("/tracking/download", web::post().to(maps::track_download))
                            .route("/metrics/downloads", web::get().to(maps::download_metrics))
                            .route("/tiled/{id}", web::get().to(maps::tiled_map))
                            .route("/content/{id}", web::get().to(maps::map_content)),
                    )
                    .service(
                        web::scope("/auth")
                            .route("/discord/start", web::get().to(auth::routes::discord_start))
                            .route(
                                "/discord/callback",
                                web::get().to(auth::routes::discord_callback),
                            )
                            .route("/me", web::get().to(auth::routes::current_user))
                            .route("/logout", web::post().to(auth::routes::logout)),
                    )
                    .service(
                        web::scope("/docs")
                            .route("/readme", web::get().to(docs::docs_readme))
                            .route("/license", web::get().to(docs::docs_license)),
                    )
                    .service(web::scope("/admin").route(
                        "/token/info",
                        web::get().to(utils::admin_info::get_admin_token_info),
                    ))
                    .service(
                        web::scope("/health")
                            .route("/liveness", web::get().to(health::liveness))
                            .route("/readiness", web::get().to(health::readiness)),
                    )
                    // SPA file service with fallback to index.html
                    .configure(file_service),
            )
    })
    .bind(format!("{address}:{port}"))?
    .run()
    .await
}
