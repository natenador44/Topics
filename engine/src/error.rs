use axum::http::StatusCode;
use axum::response::IntoResponse;
use error_stack::Report;
use std::error::Error;

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
