use crate::error::SetServiceError;
use error_stack::Report;

pub type ServiceResult<T> = Result<T, Report<SetServiceError>>;
pub type OptServiceResult<T> = Result<Option<T>, Report<SetServiceError>>;
mod error;
mod model;
pub mod repository;
pub mod routes;
pub mod service;
pub mod state;
