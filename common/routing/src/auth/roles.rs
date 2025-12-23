use std::{fmt::Display, str::FromStr};

use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use tracing::{debug, error, warn};

use crate::auth::user::AuthedUser;

pub trait Roles: FromStr + Display + Clone + Send + Sync + 'static {
    fn none() -> Self;
    fn is_none(&self) -> bool;
    fn contains(&self, other: Self) -> bool;
    fn add(&mut self, other: Self);
}

pub async fn require_roles<R: Roles>(
    State(required_roles): State<R>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let user = req.extensions().get::<AuthedUser<R>>().ok_or_else(|| {
        error!("endpoint requires authorized user, none was found");
        StatusCode::UNAUTHORIZED
    })?;

    debug!("required roles: {required_roles}");
    if required_roles.is_none() || user.has_roles(required_roles) {
        Ok(next.run(req).await)
    } else {
        warn!("User {} does not have the authority! (🧙‍♂️🚫➡️)", user.id);
        Err(StatusCode::FORBIDDEN)
    }
}
