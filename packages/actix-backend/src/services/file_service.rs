use actix_files::Files;
use actix_web::{
    HttpResponse,
    dev::{ServiceRequest, ServiceResponse},
    error::ErrorInternalServerError,
    web,
};
use std::env;
use std::path::PathBuf;

/// Configure static file serving (SPA shell fallback)
pub fn file_service(cfg: &mut web::ServiceConfig) {
    // Determine the dist directory
    let dist = env::var("DIST_DIR").map_or_else(|_| PathBuf::from("dist"), PathBuf::from);

    // Build the Files service with SPA fallback to index.html
    let files = Files::new("/", &dist)
        .index_file("index.html")
        .default_handler(move |req: ServiceRequest| {
            let dist = dist.clone();
            async move {
                // Read the SPA shell
                let html = tokio::fs::read_to_string(dist.join("index.html"))
                    .await
                    .map_err(|_| ErrorInternalServerError("Failed to read index.html"))?;

                // Reconstruct parts to build a ServiceResponse
                let (http_req, _payload) = req.into_parts();
                let resp = HttpResponse::Ok()
                    .content_type("text/html; charset=utf-8")
                    .body(html)
                    .map_into_boxed_body();

                Ok(ServiceResponse::new(http_req, resp))
            }
        });

    cfg.service(files);
}
