use actix_web::middleware::DefaultHeaders;

/// Configure security headers middleware
pub fn security() -> DefaultHeaders {
    DefaultHeaders::new()
        .add(("X-Content-Type-Options", "nosniff"))
        .add(("X-Frame-Options", "DENY"))
        .add(("Referrer-Policy", "strict-origin"))
        .add(("Content-Security-Policy", "default-src 'self'"))
}
