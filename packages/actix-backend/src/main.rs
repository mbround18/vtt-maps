mod clients;
mod docs;
mod health;
mod maps;
mod utils;

use actix_cors::Cors;
use actix_files::Files;
use actix_web::{
    App, HttpServer,
    web::{self},
};

use std::env;
use tokio::fs;
use tracing_actix_web::TracingLogger;
use utils::folders::thumbnails_dir;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    let address = env::var("ADDRESS").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let thumb_dir = thumbnails_dir().unwrap_or_else(|_| {
        panic!("Failed to get thumbnails directory");
    });
    fs::create_dir_all(&thumb_dir).await.unwrap_or_else(|_| {
        panic!("Failed to create thumbnail directory: {:?}", thumb_dir);
    });

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .wrap({
                if let Ok(origins) = env::var("CORS_ALLOWED_ORIGINS") {
                    let mut c = Cors::default().allow_any_header().allow_any_method();
                    for origin in origins.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                        c = c.allowed_origin(origin);
                    }
                    c
                } else {
                    Cors::default()
                        .allow_any_origin()
                        .allow_any_header()
                        .allow_any_method()
                }
            })
            .service(Files::new("/assets/thumbnails", thumb_dir.clone()))
            .service(
                web::scope("/api/maps")
                    .route("/all", web::get().to(maps::maps_all))
                    .route("/{id}", web::get().to(maps::map_detail))
                    .route("/rebuild", web::post().to(maps::maps_rebuild))
                    .route("/download/{id}", web::get().to(maps::download_map))
                    .route("/tiled/{id}", web::get().to(maps::tiled_map)),
            )
            .service(web::scope("/api/docs").route("/readme", web::get().to(docs::docs_readme)))
            .service(
                web::scope("/health")
                    .route("/liveness", web::get().to(health::liveness))
                    .route("/readiness", web::get().to(health::readiness)),
            )
    })
    .bind(format!("{}:{}", address, port))?
    .run()
    .await
}
