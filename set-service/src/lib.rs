
use error_stack::Report;
use crate::error::SetServiceError;

pub type ServiceResult<T> = Result<T, Report<SetServiceError>>;
pub type OptServiceResult<T> = Result<Option<T>, Report<SetServiceError>>;
pub mod repository;
pub mod routes;
pub mod service;
pub mod state;
mod error;
mod model;
