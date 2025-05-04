use gloo_net::http::{Request, RequestBuilder};

const BASE_LIMIT: u32 = 100;
const API_BASE: &str = "/api";

#[derive(Debug, Clone, PartialEq)]
pub enum ApiEndpoint {
    GetAllMaps {
        limit: Option<u32>,
        offset: Option<u32>,
    },
    GetMap {
        id: String,
    },
    GetTiledMap {
        id: String,
    },
    GetMarkdown {
        path: String,
    },
    GetMapContent {
        id: String,
    },
}

impl ApiEndpoint {
    pub fn url(&self) -> String {
        match self {
            ApiEndpoint::GetAllMaps { limit, offset } => {
                let mut params = Vec::new();
                if let Some(l) = limit {
                    params.push(format!("limit={}", l));
                } else {
                    params.push(format!("limit={}", BASE_LIMIT));
                }
                if let Some(o) = offset {
                    params.push(format!("offset={}", o));
                }
                let qs = if params.is_empty() {
                    String::new()
                } else {
                    format!("?{}", params.join("&"))
                };
                format!("{}/maps/all{}", API_BASE, qs)
            }
            ApiEndpoint::GetMap { id } => format!("{}/maps/{}", API_BASE, id),
            ApiEndpoint::GetTiledMap { id } => format!("{}/maps/tiled/{}", API_BASE, id),
            ApiEndpoint::GetMarkdown { path } => format!("{}/docs/{path}", API_BASE),
            ApiEndpoint::GetMapContent { id } => format!("{}/maps/content/{}", API_BASE, id),
        }
    }

    pub fn request(&self) -> RequestBuilder {
        match self {
            ApiEndpoint::GetAllMaps { .. }
            | ApiEndpoint::GetMap { .. }
            | ApiEndpoint::GetTiledMap { .. }
            | ApiEndpoint::GetMapContent { .. }
            | ApiEndpoint::GetMarkdown { .. } => Request::get(&self.url()),
        }
    }
}
