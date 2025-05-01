use actix_web::{HttpResponse, Responder};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    timestamp: u64,
    version: String,
}

/// Liveness probe - indicates if the application is running
pub async fn liveness() -> impl Responder {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let version = option_env!("CARGO_PKG_VERSION").unwrap_or("unknown");

    HttpResponse::Ok().json(HealthResponse {
        status: "UP".to_string(),
        timestamp,
        version: version.to_string(),
    })
}

/// Readiness probe - indicates if the application is ready to accept traffic
pub async fn readiness() -> impl Responder {
    // In a more complex application, you might check dependencies here
    // like database connections, external services, etc.
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let version = option_env!("CARGO_PKG_VERSION").unwrap_or("unknown");

    HttpResponse::Ok().json(HealthResponse {
        status: "READY".to_string(),
        timestamp,
        version: version.to_string(),
    })
}
