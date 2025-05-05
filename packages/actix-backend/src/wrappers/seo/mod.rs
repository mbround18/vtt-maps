use actix_web::{
    Error, HttpResponse,
    body::{BoxBody, to_bytes},
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::header::CONTENT_TYPE,
};
use futures_util::future::{LocalBoxFuture, Ready, ok};
use std::rc::Rc;
use tracing::instrument;

mod handle_default_metadata;
mod handle_map_details;
mod inject_seo_metadata;

use crate::wrappers::seo::{
    handle_default_metadata::handle_default_metadata,
    handle_map_details::handle_map_details_metadata,
};

pub struct SeoMetadata;

impl<S, B> Transform<S, ServiceRequest> for SeoMetadata
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = SeoMetadataMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(SeoMetadataMiddleware(Rc::new(service)))
    }
}

pub struct SeoMetadataMiddleware<S>(Rc<S>);

impl<S, B> Service<ServiceRequest> for SeoMetadataMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.0.poll_ready(ctx)
    }

    #[instrument(skip_all, fields(uri = %req.uri()))]
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = Rc::clone(&self.0);
        Box::pin(async move {
            let res = srv.call(req).await?;

            let is_html = res
                .headers()
                .get(CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .map_or(false, |ct| ct.starts_with("text/html"));

            if !is_html {
                return Ok(res.map_into_boxed_body());
            }

            // Deconstruct response and read body
            let (req, resp) = res.into_parts();
            let status = resp.status(); // Save before into_body()
            let body_bytes = to_bytes(resp.into_body())
                .await
                .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to read body"))?;

            // Process HTML
            let body_str = String::from_utf8_lossy(&body_bytes).into_owned();
            let new_body = if req.uri().path().starts_with("/maps/") {
                handle_map_details_metadata(body_str, &req).await?
            } else {
                handle_default_metadata(body_str, &req).await?
            };

            let updated_resp = HttpResponse::build(status)
                .content_type("text/html; charset=utf-8")
                .body(new_body)
                .map_into_boxed_body();

            Ok(ServiceResponse::new(req, updated_resp))
        })
    }
}
