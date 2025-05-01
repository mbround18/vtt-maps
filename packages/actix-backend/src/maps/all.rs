use crate::clients::meilisearch::meilisearch_index;
use actix_web::HttpResponse;
use actix_web::web::Query;
use serde::Deserialize;
use shared::types::map_document::MapDocument as MapDoc;
use tracing::debug;

fn default_limit() -> usize {
    10
}

fn default_offset() -> usize {
    0
}

#[derive(Deserialize, Clone, Copy)]
pub struct PaginationParams {
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default = "default_offset")]
    pub offset: usize,
}

pub async fn maps_all(query: Query<PaginationParams>) -> Result<HttpResponse, actix_web::Error> {
    let PaginationParams { limit, offset } = query.into_inner();
    debug!(
        "Request for maps_all with limit: {}, offset: {}",
        limit, offset
    );
    let index = meilisearch_index("maps")?;
    debug!("Executing search with limit: {}, offset: {}", limit, offset);
    let search = index
        .search()
        .with_limit(limit)
        .with_offset(offset)
        .execute::<MapDoc>()
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let docs: Vec<MapDoc> = search.hits.into_iter().map(|h| h.result).collect();
    debug!("Found {} maps", docs.len());
    Ok(HttpResponse::Ok().json(docs))
}
