use error_stack::Report;
use crate::error::TopicServiceError;

pub type ServiceResult<T> = Result<T, Report<TopicServiceError>>;
pub type OptServiceResult<T> = Result<Option<T>, Report<TopicServiceError>>;
pub mod repository;
pub mod routes;
pub mod service;
pub mod state;
mod error;
mod model;