use utoipa::{
    PartialSchema,
    openapi::{RefOr, Schema},
};

pub mod error;
pub mod list_criteria;
pub mod pagination;
pub mod stream;

pub fn patch_field_schema() -> impl Into<RefOr<Schema>> {
    <Option<String> as PartialSchema>::schema()
}
