use gloo_net::http::{Request, RequestBuilder};

pub const API_BASE_URL: &str = "http://127.0.0.1:8080";

pub fn api_base_url() -> String {
    std::env::var("API_BASE_URL").unwrap_or_else(|_| API_BASE_URL.to_string())
}

pub fn get(path: &str) -> RequestBuilder {
    Request::get(&format!("{}/api{}", api_base_url(), path))
}
