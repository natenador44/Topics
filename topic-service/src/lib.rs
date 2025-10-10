use crate::error::TopicServiceError;
use error_stack::Report;

pub type ServiceResult<T> = Result<T, Report<TopicServiceError>>;
pub type OptServiceResult<T> = Result<Option<T>, Report<TopicServiceError>>;
mod error;
mod model;
pub mod repository;
pub mod routes;
pub mod service;
pub mod state;

mod filter;
