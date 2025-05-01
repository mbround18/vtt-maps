use crate::clients::meilisearch::meilisearch_index;
use actix_web::{Error, HttpResponse, web};
use shared::types::map_document::MapDocument as MapDoc;
use tracing::debug;

pub async fn map_detail(id: web::Path<String>) -> Result<HttpResponse, Error> {
    let id = id.into_inner();
    debug!("Request for map detail with id: {}", id);

    let index = meilisearch_index("maps")?;

    debug!("Fetching document with id: {}", id);
    let doc = index.get_document::<MapDoc>(&id).await.map_err(|e| {
        debug!("Document not found: {}", e);
        actix_web::error::ErrorNotFound(e)
    })?;

    debug!("Found map: {}", doc.name);
    Ok(HttpResponse::Ok().json(doc))
}
