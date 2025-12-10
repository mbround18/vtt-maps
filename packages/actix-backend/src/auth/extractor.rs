use actix_web::{Error, FromRequest, HttpRequest, dev::Payload, error, web};
use futures_util::future::{Ready, ready};

use crate::auth::jwt::{JwtService, SESSION_COOKIE_NAME};
use crate::auth::models::AuthenticatedUser;

/// Extractor that validates the JWT session cookie and exposes the authenticated user.
pub struct AuthenticatedSession(pub AuthenticatedUser);

impl AuthenticatedSession {
    pub fn user(&self) -> &AuthenticatedUser {
        &self.0
    }
}

impl FromRequest for AuthenticatedSession {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let service = match req.app_data::<web::Data<JwtService>>() {
            Some(svc) => svc.clone(),
            None => {
                return ready(Err(error::ErrorInternalServerError(
                    "JWT service unavailable",
                )));
            }
        };

        let cookie = match req.cookie(SESSION_COOKIE_NAME) {
            Some(cookie) => cookie,
            None => return ready(Err(error::ErrorUnauthorized("missing session"))),
        };

        match service.verify(cookie.value()) {
            Ok(user) => ready(Ok(AuthenticatedSession(user))),
            Err(_) => ready(Err(error::ErrorUnauthorized("invalid session"))),
        }
    }
}
