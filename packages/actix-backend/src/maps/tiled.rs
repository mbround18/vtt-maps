use actix_web::{Error, HttpResponse, web};
use base64::engine::{Engine, general_purpose};
use bytes::Bytes;
use futures::stream::once;
use shared::types::map_document::MapDocument as MapDoc;
use shared::utils::root_dir::root_dir;
use std::fs;
use std::path::Path;
use tracing::{debug, error};

use crate::clients::meilisearch::meilisearch_index;

pub async fn tiled_map(id: web::Path<String>) -> Result<HttpResponse, Error> {
    let id = id.into_inner();
    debug!("Request for tiled map with id: {}", id);

    // Get the map document from MeiliSearch
    let index = meilisearch_index("maps")?;
    let doc = match index.get_document::<MapDoc>(&id).await {
        Ok(doc) => doc,
        Err(e) => {
            error!("Failed to find map document: {}", e);
            return Ok(HttpResponse::NotFound().body(format!("Map with ID {} not found", id)));
        }
    };

    // Get the path to the file
    let root_dir = root_dir()
        .map_err(Error::from)?
        .to_string_lossy()
        .to_string();
    let dd2vtt_path = format!("{}{}", root_dir, &doc.path);

    let path = Path::new(&dd2vtt_path);
    if !path.exists() {
        error!("Map file not found at path: {}", doc.path);
        return Ok(HttpResponse::NotFound().body("Map file not found"));
    }

    // Load and parse the DD2VTT file
    match fs::File::open(path) {
        Ok(file) => {
            // Parse the DD2VTT file to extract the base64 image
            match serde_json::from_reader::<_, shared::types::dd2vtt::DD2VTTFile>(file) {
                Ok(dd2vtt) => {
                    // Decode the base64 image using the non-deprecated method
                    match general_purpose::STANDARD.decode(&dd2vtt.image) {
                        Ok(image_bytes) => {
                            // Create a stream from the image bytes
                            let stream =
                                once(async move { Ok::<_, Error>(Bytes::from(image_bytes)) });

                            // Return the image with proper content type header
                            Ok(HttpResponse::Ok()
                                .content_type("image/png")
                                .streaming(stream))
                        }
                        Err(e) => {
                            error!("Failed to decode base64 image: {}", e);
                            Ok(HttpResponse::InternalServerError()
                                .body("Failed to decode map image"))
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to parse DD2VTT file: {}", e);
                    Ok(HttpResponse::InternalServerError().body("Failed to parse DD2VTT file"))
                }
            }
        }
        Err(e) => {
            error!("Failed to open DD2VTT file: {}", e);
            Ok(HttpResponse::InternalServerError().body("Failed to read DD2VTT file"))
        }
    }
}
