use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

pub type TopicId = Uuid;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Topic {
    id: Uuid,
    name: String,
    description: String,
}
