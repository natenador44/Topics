use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

pub type TopicId = Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct Topic {
    pub id: Uuid,
    pub name: String,
    pub description: String,
}
