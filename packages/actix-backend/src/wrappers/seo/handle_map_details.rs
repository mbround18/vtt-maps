use crate::clients::meilisearch::meilisearch_index;
use crate::wrappers::seo::inject_seo_metadata::{SeoData, inject_seo_metadata};
use actix_web::error::ErrorNotFound;
use actix_web::{Error, HttpRequest};
use shared::types::map_document::MapDocument;

pub(crate) async fn handle_map_details_metadata(
    html: String,
    http_request: &HttpRequest,
) -> Result<String, Error> {
    // Extract the map ID from the url which is in the format "/maps/{id}"
    let uri = http_request
        .uri()
        .path()
        .strip_prefix("/maps/")
        .unwrap_or_default();

    let id = uri.split('/').next_back().unwrap_or_default();

    let doc = meilisearch_index("maps")?
        .get_document::<MapDocument>(id)
        .await
        .map_err(|_| ErrorNotFound("Map metadata not found"))?;

    let seo = SeoData {
        title: format!("{} | D&D VTT Maps", doc.name),
        description: format!("View the {} battle map on D&D VTT Maps", doc.name),
        keywords: Some(format!("D&D, VTT, Maps, {}", doc.name)),
        image_url: doc.thumbnail.clone(),
    };
    inject_seo_metadata(html, http_request, seo)
}
