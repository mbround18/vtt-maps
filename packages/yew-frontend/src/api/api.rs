use gloo_net::http::{Request, RequestBuilder};

const BASE_LIMIT: u32 = 100;
const API_BASE: &str = "/api";

#[derive(Debug, Clone)]
pub enum ApiEndpoint {
    GetAllMaps {limit: Option<u32>, offset: Option<u32>},
    GetMap { id: String },
    // RebuildMaps,
    DownloadMap { id: String },
    GetTiledMap { id: String },
    GetReadme,
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
            // ApiEndpoint::RebuildMaps => format!("{}/maps/rebuild", API_BASE),
            ApiEndpoint::DownloadMap { id } => format!("{}/maps/download/{}", API_BASE, id),
            ApiEndpoint::GetTiledMap { id } => format!("{}/maps/tiled/{}", API_BASE, id),
            ApiEndpoint::GetReadme => format!("{}/docs/readme", API_BASE),
        }
    }

    pub fn request(&self) -> RequestBuilder {
        match self {
            ApiEndpoint::GetAllMaps { .. }
            | ApiEndpoint::GetMap { .. }
            | ApiEndpoint::DownloadMap { .. }
            | ApiEndpoint::GetTiledMap { .. }
            | ApiEndpoint::GetReadme => Request::get(&self.url()),
            // ApiEndpoint::RebuildMaps => Request::post(&self.url()),
        }
    }
}
