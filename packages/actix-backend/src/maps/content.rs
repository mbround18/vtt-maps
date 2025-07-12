use crate::clients::meilisearch::meilisearch_index;
use crate::docs::serve_markdown_file;
use actix_web::{Error, HttpResponse, web};
use shared::types::map_document::MapDocument as MapDoc;
use shared::utils::root_dir::root_dir;
use std::path::Path;
use tracing::debug;

pub async fn map_content(id: web::Path<String>) -> Result<HttpResponse, Error> {
    let id = id.into_inner();
    debug!("Request for map detail with id: {}", id);
    let index = meilisearch_index("maps")?;

    debug!("Fetching document with id: {}", id);
    let doc: MapDoc = index.get_document::<MapDoc>(&id).await.map_err(|e| {
        debug!("Document not found: {}", e);
        actix_web::error::ErrorNotFound(e)
    })?;
    if let Some(content) = doc.content.clone() {
        debug!("Document found: {}", doc.name);

        let root = root_dir()?;
        let content_full_path = format!("{}/{}", root.to_string_lossy(), content);
        let content_path = Path::new(&content_full_path);
        debug!("Content path: {:?}", content_path);
        if content_path.exists() {
            serve_markdown_file(content_path).await
        } else {
            debug!("Content file not found: {}", content);
            Ok(HttpResponse::NotFound().body("Map content not found"))
        }
    } else {
        debug!("No content found for document: {}", doc.name);
        Ok(HttpResponse::NotFound().body("Map content not found"))
    }
}
