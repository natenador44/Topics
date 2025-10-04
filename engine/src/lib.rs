use crate::repository::TopicsRepository;
use std::fmt::Debug;
use utoipa::openapi::{RefOr, Schema};
use utoipa::PartialSchema;

pub mod error;
pub mod models;
mod pagination;
pub use pagination::Pagination;
pub mod repository;
pub mod search_criteria;
pub mod search_filters;

pub trait Engine: Debug + Clone + Send + Sync + 'static {
    type Repo: TopicsRepository + Send + Sync + 'static;
    // type Cache: Cache + Send + Sync + 'static;

    fn topics(&self) -> Self::Repo;
}

pub fn patch_field_schema() -> impl Into<RefOr<Schema>> {
    <Option<String> as PartialSchema>::schema()
}
