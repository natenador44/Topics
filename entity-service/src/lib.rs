
use error_stack::Report;
use crate::error::EntityServiceError;

pub type ServiceResult<T> = Result<T, Report<EntityServiceError>>;
pub type OptServiceResult<T> = Result<Option<T>, Report<EntityServiceError>>;
pub mod repository;
pub mod routes;
pub mod service;
pub mod state;
mod error;
mod model;
