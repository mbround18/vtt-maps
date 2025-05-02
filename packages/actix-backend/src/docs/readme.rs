use actix_web::{HttpResponse, error::ErrorInternalServerError};
use shared::utils::root_dir::root_dir;
use tokio::fs;
use tracing::{debug, error};

use crate::utils::markdown::markdown_to_html;

pub async fn docs_readme() -> Result<HttpResponse, actix_web::Error> {
    debug!("Request for README documentation");

    let root = root_dir().map_err(ErrorInternalServerError)?;
    let readme_path = root.join("README.md");

    let markdown_content = fs::read_to_string(&readme_path).await.map_err(|e| {
        error!("Failed to read README.md: {}", e);
        ErrorInternalServerError("Failed to read README file")
    })?;

    let html_content = markdown_to_html(&markdown_content);

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html_content))
}
