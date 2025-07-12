use actix_cors::Cors;

pub fn cors() -> Cors {
    if let Ok(origins) = std::env::var("CORS_ALLOWED_ORIGINS") {
        let mut c = Cors::default().allow_any_method().allow_any_header();
        for origin in origins.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            c = c.allowed_origin(origin);
        }
        c.supports_credentials()
    } else {
        Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials()
    }
}
