use utoipa::PartialSchema;
use utoipa::openapi::{RefOr, Schema};

pub mod error;
mod pagination;
pub use pagination::Pagination;
pub mod list_criteria;
pub mod stream;

pub mod id;

pub mod app;

pub fn patch_field_schema() -> impl Into<RefOr<Schema>> {
    <Option<String> as PartialSchema>::schema()
}
