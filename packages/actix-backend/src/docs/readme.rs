use crate::docs::serve_markdown::serve_markdown_file;
use actix_web::{HttpResponse, error::ErrorInternalServerError};
use shared::utils::root_dir::root_dir;

pub async fn docs_readme() -> Result<HttpResponse, actix_web::Error> {
    let root = root_dir().map_err(ErrorInternalServerError)?;
    let license = root.join("README.md");
    serve_markdown_file(license).await
}
