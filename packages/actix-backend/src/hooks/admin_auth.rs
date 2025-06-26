use crate::utils::admin_token::validate_admin_token;
use actix_web::{
    Error, HttpMessage, HttpRequest, HttpResponse,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::header::AUTHORIZATION,
};
use futures_util::future::{LocalBoxFuture, Ready, ok};
use serde_json::json;
use std::rc::Rc;
use tracing::{debug, warn};

/// Middleware for admin token authentication.
///
/// Expects the admin token to be provided in one of these ways:
/// 1. Authorization header: `Bearer <token>`
/// 2. Authorization header: `<token>` (without Bearer prefix)
/// 3. Query parameter: `?admin_token=<token>`
pub struct AdminAuth;

impl<S, B> Transform<S, ServiceRequest> for AdminAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<actix_web::body::EitherBody<B>>;
    type Error = Error;
    type Transform = AdminAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AdminAuthMiddleware(Rc::new(service)))
    }
}

pub struct AdminAuthMiddleware<S>(Rc<S>);

impl<S, B> Service<ServiceRequest> for AdminAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<actix_web::body::EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.0.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = Rc::clone(&self.0);

        Box::pin(async move {
            let token = extract_admin_token(&req);

            match token {
                Some(provided_token) => {
                    match validate_admin_token(&provided_token) {
                        Ok(true) => {
                            debug!("✅ Admin token validated successfully");
                            // Store admin flag in request extensions for later use
                            req.extensions_mut().insert(AdminAuthenticated);
                            let res = srv.call(req).await?;
                            Ok(res.map_into_left_body())
                        }
                        Ok(false) => {
                            warn!("❌ Invalid admin token provided");
                            let (req, _) = req.into_parts();
                            let response = HttpResponse::Forbidden().json(json!({
                                "error": "Invalid admin token",
                                "code": "INVALID_ADMIN_TOKEN"
                            }));
                            Ok(ServiceResponse::new(req, response).map_into_right_body())
                        }
                        Err(e) => {
                            warn!("❌ Error validating admin token: {}", e);
                            let (req, _) = req.into_parts();
                            let response = HttpResponse::InternalServerError().json(json!({
                                "error": "Authentication service unavailable",
                                "code": "AUTH_SERVICE_ERROR"
                            }));
                            Ok(ServiceResponse::new(req, response).map_into_right_body())
                        }
                    }
                }
                None => {
                    warn!("❌ No admin token provided for protected endpoint");
                    let (req, _) = req.into_parts();
                    let response = HttpResponse::Unauthorized()
                        .json(json!({
                            "error": "Admin token required",
                            "code": "ADMIN_TOKEN_REQUIRED",
                            "hint": "Provide admin token via 'Authorization: Bearer <token>' header or '?admin_token=<token>' query parameter"
                        }));
                    Ok(ServiceResponse::new(req, response).map_into_right_body())
                }
            }
        })
    }
}

/// Marker type to indicate admin authentication in request extensions
#[derive(Clone)]
pub struct AdminAuthenticated;

/// Extracts admin token from the request.
///
/// Checks in order:
/// 1. Authorization header (Bearer token)
/// 2. Authorization header (direct token)
/// 3. Query parameter `admin_token`
fn extract_admin_token(req: &ServiceRequest) -> Option<String> {
    // Check Authorization header first
    if let Some(auth_header) = req.headers().get(AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            // Try Bearer token format
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                return Some(token.trim().to_string());
            }
            // Try direct token format
            if !auth_str.is_empty() {
                return Some(auth_str.trim().to_string());
            }
        }
    }

    // Check query parameter
    if let Some(query_str) = req.uri().query() {
        for pair in query_str.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                if key == "admin_token" {
                    return Some(value.to_string());
                }
            }
        }
    }

    None
}

/// Helper function to check if a request is admin authenticated
#[allow(dead_code)]
pub fn is_admin_authenticated(req: &HttpRequest) -> bool {
    req.extensions().get::<AdminAuthenticated>().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;

    #[actix_web::test]
    async fn test_extract_admin_token_from_bearer_header() {
        let req = test::TestRequest::default()
            .append_header(("Authorization", "Bearer test_token_123"))
            .to_srv_request();

        let token = extract_admin_token(&req);
        assert_eq!(token, Some("test_token_123".to_string()));
    }

    #[actix_web::test]
    async fn test_extract_admin_token_from_direct_header() {
        let req = test::TestRequest::default()
            .append_header(("Authorization", "test_token_456"))
            .to_srv_request();

        let token = extract_admin_token(&req);
        assert_eq!(token, Some("test_token_456".to_string()));
    }

    #[actix_web::test]
    async fn test_extract_admin_token_from_query() {
        let req = test::TestRequest::default()
            .uri("/test?admin_token=query_token_789&other=param")
            .to_srv_request();

        let token = extract_admin_token(&req);
        assert_eq!(token, Some("query_token_789".to_string()));
    }

    #[actix_web::test]
    async fn test_extract_admin_token_none() {
        let req = test::TestRequest::default().uri("/test").to_srv_request();

        let token = extract_admin_token(&req);
        assert_eq!(token, None);
    }
}
