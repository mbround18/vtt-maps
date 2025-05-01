use actix_web::{
    Error, HttpResponse,
    error::{ErrorInternalServerError, ErrorNotFound},
    web,
};
use meilisearch_sdk::client::Client;
use shared::types::map_document::MapDocument as MapDoc;
use shared::utils::root_dir::root_dir;
use std::env;
use tokio::fs;
use tracing::{debug, error};

pub async fn download_map(id: web::Path<String>) -> Result<HttpResponse, Error> {
    let id = id.into_inner();
    debug!("Request for map download with id: {}", id);

    let url = env::var("MEILI_URL").unwrap_or_else(|_| "http://127.0.0.1:7700".into());
    let key = env::var("MEILI_KEY").unwrap_or_default();
    let client = Client::new(&url, Some(&key)).map_err(ErrorInternalServerError)?;
    let index = client.index("maps");

    let doc = index.get_document::<MapDoc>(&id).await.map_err(|e| {
        debug!("Document metadata not found: {}", e);
        ErrorNotFound("Map metadata not found")
    })?;

    debug!("Found map metadata: {:?}", &doc);

    let root_path = root_dir().map_err(ErrorInternalServerError)?;
    let canonical_path = fs::canonicalize(format!("{}/{}", root_path.display(), doc.path))
        .await
        .map_err(|e| {
            error!(
                "Failed to canonicalize file path: {}\n{:?}/{}",
                e, root_path, doc.path
            );
            ErrorInternalServerError("Failed to canonicalize file path of request map.")
        })?;

    let data = fs::read(&canonical_path).await.map_err(|e| {
        debug!(
            "Map file not found or unreadable: {}\n\tFile Path: {:?}",
            e, canonical_path
        );
        ErrorNotFound("Map file not found")
    })?;

    let filename = canonical_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("map.dd2vtt");

    Ok(HttpResponse::Ok()
        .content_type("application/octet-stream")
        .append_header((
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", filename),
        ))
        .body(data))
}
