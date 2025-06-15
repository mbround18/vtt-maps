use actix_web::{Error, HttpResponse, web};
use base64::engine::{Engine, general_purpose};
use bytes::Bytes;
use shared::types::map_document::MapDocument as MapDoc;
use shared::utils::root_dir::root_dir;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, error};

use crate::clients::meilisearch::meilisearch_index;

async fn get_map_document(id: &str) -> Result<MapDoc, HttpResponse> {
    let index = meilisearch_index("maps").map_err(|e| {
        error!("Failed to get meilisearch index: {}", e);
        HttpResponse::InternalServerError().body("Search service unavailable")
    })?;

    index.get_document::<MapDoc>(id).await.map_err(|e| {
        error!("Failed to find map document: {}", e);
        HttpResponse::NotFound().body(format!("Map with ID {} not found", id))
    })
}

fn build_file_path(doc: &MapDoc) -> Result<PathBuf, HttpResponse> {
    let root_dir = root_dir().map_err(|e| {
        error!("Failed to get root directory: {}", e);
        HttpResponse::InternalServerError().body("Configuration error")
    })?;

    let full_path = root_dir.join(doc.path.trim_start_matches('/'));

    if !full_path.exists() {
        error!("Map file not found at apath: {}", doc.path);
        return Err(HttpResponse::NotFound().body("Map file not found"));
    }

    Ok(full_path)
}

fn load_and_decode_image(path: &Path) -> Result<Bytes, HttpResponse> {
    let file = fs::File::open(path).map_err(|e| {
        error!("Failed to open DD2VTT file: {}", e);
        HttpResponse::InternalServerError().body("Failed to read DD2VTT file")
    })?;

    let dd2vtt: shared::types::dd2vtt::DD2VTTFile = serde_json::from_reader(file).map_err(|e| {
        error!("Failed to parse DD2VTT file: {}", e);
        HttpResponse::InternalServerError().body("Failed to parse DD2VTT file")
    })?;

    let image_bytes = general_purpose::STANDARD
        .decode(&dd2vtt.image)
        .map_err(|e| {
            error!("Failed to decode base64 image: {}", e);
            HttpResponse::InternalServerError().body("Failed to decode map image")
        })?;

    Ok(Bytes::from(image_bytes))
}

pub async fn tiled_map(id: web::Path<String>) -> Result<HttpResponse, Error> {
    let id = id.into_inner();
    debug!("Request for tiled map with id: {}", id);

    let doc = match get_map_document(&id).await {
        Ok(doc) => doc,
        Err(response) => return Ok(response),
    };

    let file_path = match build_file_path(&doc) {
        Ok(path) => path,
        Err(response) => return Ok(response),
    };

    let image_data = match load_and_decode_image(&file_path) {
        Ok(data) => data,
        Err(response) => return Ok(response),
    };

    Ok(HttpResponse::Ok()
        .content_type("image/png")
        .body(image_data))
}
