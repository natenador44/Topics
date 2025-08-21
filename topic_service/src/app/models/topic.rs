use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
pub struct TopicId {
    #[schema(value_type=String)]
    inner: Uuid,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Topic {
    id: TopicId,
    name: String,
    description: String,
}
