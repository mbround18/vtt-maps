use crate::clients::meilisearch::meilisearch_index;
use actix_web::{
    Error, HttpResponse,
    error::{ErrorInternalServerError, ErrorNotFound},
    web,
};
use shared::types::map_document::MapDocument as MapDoc;
use shared::utils::root_dir::root_dir;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, error};

async fn retrieve_map_document(id: &str) -> Result<MapDoc, Error> {
    let index = meilisearch_index("maps")?;

    index.get_document::<MapDoc>(id).await.map_err(|e| {
        debug!("Document metadata not found: {}", e);
        ErrorNotFound("Map metadata not found")
    })
}

async fn construct_file_path(doc: &MapDoc) -> Result<PathBuf, Error> {
    let root_path = root_dir().map_err(ErrorInternalServerError)?;
    let file_path = root_path.join(doc.path.trim_start_matches('/'));

    fs::canonicalize(&file_path).await.map_err(|e| {
        error!(
            "Failed to canonicalize file path: {}\n{:?}/{}",
            e, root_path, doc.path
        );
        ErrorInternalServerError("Failed to canonicalize file path of request map.")
    })
}

async fn read_map_file(path: &Path) -> Result<Vec<u8>, Error> {
    fs::read(path).await.map_err(|e| {
        debug!(
            "Map file not found or unreadable: {}\n\tFile Path: {:?}",
            e, path
        );
        ErrorNotFound("Map file not found")
    })
}

fn extract_filename(path: &Path) -> &str {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("map.dd2vtt")
}

fn create_download_response(data: Vec<u8>, filename: &str) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("application/octet-stream")
        .append_header((
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", filename),
        ))
        .body(data)
}

pub async fn download_map(id: web::Path<String>) -> Result<HttpResponse, Error> {
    let id = id.into_inner();
    debug!("Request for map download with id: {}", id);

    let doc = retrieve_map_document(&id).await?;
    debug!("Found map metadata: {:?}", &doc);

    let canonical_path = construct_file_path(&doc).await?;
    let data = read_map_file(&canonical_path).await?;
    let filename = extract_filename(&canonical_path);

    Ok(create_download_response(data, filename))
}
