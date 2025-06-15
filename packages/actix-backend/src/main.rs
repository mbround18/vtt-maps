mod clients;
mod docs;
mod health;
mod hooks;
mod maps;
mod services;
mod utils;
mod wrappers;

use actix_files::Files;
use actix_web::dev::Service;
use actix_web::{App, HttpServer, http::header::CONTENT_TYPE, web};
use tracing::{error, info};

use crate::hooks::{cors, identity, logger::setup_logger, security};
use crate::maps::rebuild::rebuild_maps_core;
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

    let address = env::var("ADDRESS").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let root = root_dir()?;
    let thumb_dir = thumbnails_dir()?;
    match setup_folders() {
        Ok(_) => info!("Base setup complete."),
        Err(e) => {
            eprintln!("Error when checking for base setup!: {:?}", e);
            std::process::exit(1);
        }
    }

    info!("Operating out of directory: {}", root.display());

    // Run map rebuild process during initialization
    info!("🔧 Initializing maps rebuild process...");
    match rebuild_maps_core().await {
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
                eprintln!("Map rebuild failed: {:?}", e);
                std::process::exit(1);
            }
        }
    }

    info!("Listening on {}:{}", &address, &port);

    HttpServer::new(move || {
        App::new()
            // Register middleware via configure hooks
            .wrap(TracingLogger::default())
            .wrap(IdentityMiddleware::default())
            .wrap(identity::session_middleware())
            .wrap(cors::cors())
            .wrap(security::security())
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
                            .route("/download/{id}", web::get().to(maps::download_map))
                            .route("/tiled/{id}", web::get().to(maps::tiled_map))
                            .route("/content/{id}", web::get().to(maps::map_content)),
                    )
                    .service(
                        web::scope("/docs")
                            .route("/readme", web::get().to(docs::docs_readme))
                            .route("/license", web::get().to(docs::docs_license)),
                    ),
            )
            // Health checks
            .service(
                web::scope("/health")
                    .route("/liveness", web::get().to(health::liveness))
                    .route("/readiness", web::get().to(health::readiness)),
            )
            // SPA file service with fallback
            .configure(file_service)
    })
    .bind(format!("{}:{}", address, port))?
    .run()
    .await
}
