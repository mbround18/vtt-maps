use actix_web::{
    Error, HttpResponse,
    body::{BoxBody, to_bytes},
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::header::CONTENT_TYPE,
};
use futures_util::future::{LocalBoxFuture, Ready, ok};
use std::rc::Rc;
mod handle_default_metadata;
mod handle_map_details;
mod inject_seo_metadata;

use crate::wrappers::seo::handle_default_metadata::handle_default_metadata;
use crate::wrappers::seo::handle_map_details::handle_map_details_metadata;

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

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = Rc::clone(&self.0);
        Box::pin(async move {
            // call inner service
            let res = srv.call(req).await?;

            // grab status & headers before consuming body
            let status = res.status();
            let headers = res.headers().clone();

            // only transform HTML
            let is_html = headers
                .get(CONTENT_TYPE)
                .is_some_and(|ct| ct.to_str().is_ok_and(|ctx| ctx.starts_with("text/html")));

            if is_html {
                // consume original body
                let (req, resp) = res.into_parts();
                let bytes = to_bytes(resp.into_body())
                    .await
                    .map_err(|_| actix_web::error::ErrorInternalServerError("Body read error"))?;
                let mut body_str = String::from_utf8_lossy(&bytes).into_owned();

                // inject SEO
                if req.uri().path().starts_with("/maps/") {
                    body_str = handle_map_details_metadata(body_str, &req).await?;
                } else {
                    body_str = handle_default_metadata(body_str, &req).await?;
                }

                // build new HTML response, preserving status
                let new_resp = HttpResponse::build(status)
                    .content_type("text/html; charset=utf-8")
                    .body(body_str)
                    .map_into_boxed_body();

                Ok(ServiceResponse::new(req, new_resp))
            } else {
                // non-HTML: just re-box and return original
                Ok(res.map_into_boxed_body())
            }
        })
    }
}
