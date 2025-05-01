use actix_web::Error;
use actix_web::error::ErrorInternalServerError;
use meilisearch_sdk::{client::Client, indexes::Index};
use std::env;

pub fn meilisearch_index(name: &str) -> Result<Index, Error> {
    let url = env::var("MEILI_URL").unwrap_or_else(|_| "http://127.0.0.1:7700".into());
    let key = env::var("MEILI_KEY").unwrap_or_default();
    let client = Client::new(&url, Some(&key)).map_err(ErrorInternalServerError)?;
    Ok(client.index(name))
}
