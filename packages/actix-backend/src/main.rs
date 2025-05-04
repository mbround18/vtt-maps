mod clients;
mod docs;
mod health;
mod maps;
mod utils;
mod wrappers;

use crate::wrappers::seo::SeoMetadata;
use actix_cors::Cors;
use actix_files::Files;
use actix_web::body::BoxBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::error::ErrorInternalServerError;
use actix_web::{
    App, HttpResponse, HttpServer,
    web::{self},
};
use shared::utils::root_dir::root_dir;
use std::env;
use std::path::PathBuf;
use tracing_actix_web::TracingLogger;
use utils::folders::thumbnails_dir;
use utils::setup::setup_folders;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    setup_folders().unwrap_or_else(|e| {
        eprintln!("Error setting up folders: {:?}", e);
        std::process::exit(1);
    });

    let address = env::var("ADDRESS").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let root_dir = root_dir()?;
    let thumb_dir = thumbnails_dir()?;

    HttpServer::new(move || {
        let dist_dir = {
            env::var("DIST_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| root_dir.join("dist"))
        };

        let files_service = Files::new("/", &dist_dir)
            .index_file("index.html")
            .default_handler(move |req: ServiceRequest| {
                // capture dist_dir for the async block
                let dist = dist_dir.clone();
                async move {
                    // 1) read index.html
                    let html = tokio::fs::read_to_string(dist.join("index.html"))
                        .await
                        .map_err(|_| ErrorInternalServerError("Failed to read index.html"))?;

                    // 2) split ServiceRequest into HttpRequest + payload (we ignore payload here)
                    let (http_req, _payload) = req.into_parts();

                    // 3) build an HttpResponse<BoxBody>
                    let resp: HttpResponse<BoxBody> = HttpResponse::Ok()
                        .content_type("text/html; charset=utf-8")
                        .body(html)
                        .map_into_boxed_body();

                    // 4) wrap it into a ServiceResponse and return
                    Ok::<ServiceResponse<BoxBody>, _>(ServiceResponse::new(http_req, resp))
                }
            });

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
            .wrap(SeoMetadata)
            .service(Files::new("/assets/thumbnails", thumb_dir.clone()))
            .service(
                web::scope("/api/maps")
                    .route("/all", web::get().to(maps::maps_all))
                    .route("/{id}", web::get().to(maps::map_detail))
                    .route("/rebuild", web::post().to(maps::maps_rebuild))
                    .route("/download/{id}", web::get().to(maps::download_map))
                    .route("/tiled/{id}", web::get().to(maps::tiled_map))
                    .route("/content/{id}", web::get().to(maps::map_content)),
            )
            .service(
                web::scope("/api/docs")
                    .route("/readme", web::get().to(docs::docs_readme))
                    .route("/license", web::get().to(docs::docs_license)),
            )
            .service(
                web::scope("/health")
                    .route("/liveness", web::get().to(health::liveness))
                    .route("/readiness", web::get().to(health::readiness)),
            )
            .service(files_service)
    })
    .bind(format!("{}:{}", address, port))?
    .run()
    .await
}
