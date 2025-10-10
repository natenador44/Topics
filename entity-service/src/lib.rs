use crate::error::EntityServiceError;
use error_stack::Report;

pub type ServiceResult<T> = Result<T, Report<EntityServiceError>>;
pub type OptServiceResult<T> = Result<Option<T>, Report<EntityServiceError>>;
mod error;
mod model;
pub mod repository;
pub mod routes;
pub mod service;
pub mod state;
