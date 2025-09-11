use axum::{http::StatusCode, response::IntoResponse};

#[derive(Debug, thiserror::Error)]
pub enum InitError {
    #[error("failed to initialize logging")]
    Logging,
    #[error("failed to initialize port")]
    Port,
    #[error("failed to intialize and serve routes")]
    Serve,
}

#[derive(thiserror::Error, Debug)]
#[error("there was an error running the endpoint")]
pub struct ServiceError;

impl IntoResponse for ServiceError {
    fn into_response(self) -> axum::response::Response {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TopicServiceError {
    #[error("failed testing stuff")]
    Test,
}
