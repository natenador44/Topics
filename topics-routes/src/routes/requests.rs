use engine::patch_field_schema;
use optional_field::{Field, serde_optional_fields};
use serde::Deserialize;
use utoipa::ToSchema;

#[serde_optional_fields]
#[derive(Debug, Deserialize, ToSchema)]
pub struct TopicPatchRequest {
    /// The new name of the topic. Cannot be null. If set to null or not specified, no update will happen.
    pub name: Option<String>,
    /// The new description of the topic. Can be null. If specified as null, the description will update to null.
    /// If not specified, no update will happen.
    #[schema(schema_with = patch_field_schema)]
    pub description: Field<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct TopicSearch {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTopicRequest {
    pub name: String,
    pub description: Option<String>,
}

#[serde_optional_fields]
#[derive(Debug, Deserialize, ToSchema)]
pub struct BulkCreateTopicRequest {
    #[schema(schema_with = patch_field_schema)]
    pub name: Field<String>,
    #[schema(schema_with = patch_field_schema)]
    pub description: Field<String>,
}
