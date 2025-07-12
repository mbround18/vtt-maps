use gloo_net::http::{Request, RequestBuilder};

const BASE_LIMIT: u32 = 100;
const API_BASE: &str = "/api";

#[derive(Debug, Clone, PartialEq)]
pub enum Endpoint {
    AllMaps {
        limit: Option<u32>,
        offset: Option<u32>,
    },
    Map {
        id: String,
    },
    TiledMap {
        id: String,
    },
    Markdown {
        path: String,
    },
    MapContent {
        id: String,
    },
}

impl Endpoint {
    pub fn url(&self) -> String {
        match self {
            Endpoint::AllMaps { limit, offset } => {
                let mut params = Vec::new();
                if let Some(l) = limit {
                    params.push(format!("limit={l}"));
                } else {
                    params.push(format!("limit={BASE_LIMIT}"));
                }
                if let Some(o) = offset {
                    params.push(format!("offset={o}"));
                }
                let qs = if params.is_empty() {
                    String::new()
                } else {
                    format!("?{}", params.join("&"))
                };
                format!("{API_BASE}/maps/all{qs}")
            }
            Endpoint::Map { id } => format!("{API_BASE}/maps/{id}"),
            Endpoint::TiledMap { id } => format!("{API_BASE}/maps/tiled/{id}"),
            Endpoint::Markdown { path } => format!("{API_BASE}/docs/{path}"),
            Endpoint::MapContent { id } => format!("{API_BASE}/maps/content/{id}"),
        }
    }

    pub fn request(&self) -> RequestBuilder {
        match self {
            Endpoint::AllMaps { .. }
            | Endpoint::Map { .. }
            | Endpoint::TiledMap { .. }
            | Endpoint::MapContent { .. }
            | Endpoint::Markdown { .. } => Request::get(&self.url()),
        }
    }
}

pub use Endpoint as ApiEndpoint;
