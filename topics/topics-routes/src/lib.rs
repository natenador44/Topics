use crate::error::TopicServiceError;
use error_stack::Report;

pub type ServiceResult<T> = Result<T, Report<TopicServiceError>>;
pub type OptServiceResult<T> = Result<Option<T>, Report<TopicServiceError>>;
mod auth;
mod error;
mod metrics;
pub mod routes;
pub mod service;
pub mod state;
