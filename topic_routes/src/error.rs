use axum::{http::StatusCode, response::IntoResponse};
use error_stack::Report;
use std::error::Error;

pub type AppResult<T, E> = Result<T, Report<E>>;

#[derive(Debug, thiserror::Error)]
pub enum InitError {
    #[error("failed to initialize logging")]
    Logging,
    #[error("failed to initialize port")]
    Port,
    #[error("failed to initialize service")]
    Service,
    #[error("failed to intialize and serve routes")]
    Serve,
}

#[derive(thiserror::Error)]
#[error("there was an error running the endpoint")]
pub struct ServiceError<T: Error>(Report<T>);

impl<T: Error> std::fmt::Debug for ServiceError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> From<Report<T>> for ServiceError<T>
where
    T: Error,
{
    fn from(value: Report<T>) -> Self {
        Self(value)
    }
}

impl<T: Error> IntoResponse for ServiceError<T> {
    fn into_response(self) -> axum::response::Response {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

#[derive(Debug, thiserror::Error)]
#[error("topic service failed")]
pub struct TopicServiceError;

#[derive(Debug, thiserror::Error)]
#[error("set service failed")]
pub struct SetServiceError;

#[derive(Debug, thiserror::Error)]
#[error("entity service failed")]
pub struct EntityServiceError;
