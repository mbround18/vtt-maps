// src/utils/markdown_server.rs
use crate::utils::markdown::markdown_to_html;
use actix_web::{HttpResponse, error::ErrorInternalServerError};
use std::path::Path;
use tokio::fs;
use tracing::{debug, error};

/// Serve any local Markdown file as HTML
pub async fn serve_markdown_file(path: impl AsRef<Path>) -> Result<HttpResponse, actix_web::Error> {
    let path = path.as_ref();
    debug!("Serving markdown file: {}", path.display());

    let md = fs::read_to_string(path).await.map_err(|e| {
        error!("Failed to read {}: {}", path.display(), e);
        ErrorInternalServerError("Failed to read markdown file")
    })?;

    let html = markdown_to_html(&md);
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}
