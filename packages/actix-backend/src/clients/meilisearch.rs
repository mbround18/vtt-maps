use actix_web::Error;
use actix_web::error::ErrorInternalServerError;
use meilisearch_sdk::{client::Client, indexes::Index};
use std::{env, sync::OnceLock};

static MEILISEARCH_CLIENT: OnceLock<Client> = OnceLock::new();

fn get_meilisearch_client() -> Result<&'static Client, Error> {
    if let Some(client) = MEILISEARCH_CLIENT.get() {
        return Ok(client);
    }

    let url = env::var("MEILI_URL").unwrap_or_else(|_| "http://127.0.0.1:7700".into());
    let key = env::var("MEILI_KEY").ok();
    let client = Client::new(&url, key.as_deref()).map_err(ErrorInternalServerError)?;

    match MEILISEARCH_CLIENT.set(client) {
        Ok(()) => MEILISEARCH_CLIENT
            .get()
            .ok_or_else(|| ErrorInternalServerError("Failed to retrieve cached client")),
        Err(_) => MEILISEARCH_CLIENT
            .get()
            .ok_or_else(|| ErrorInternalServerError("Failed to cache client")),
    }
}

pub fn meilisearch_index(name: &str) -> Result<Index, Error> {
    let client = get_meilisearch_client()?;
    Ok(client.index(name))
}
