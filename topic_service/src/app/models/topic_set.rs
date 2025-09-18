use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

pub type TopicSetId = Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TopicSet {
    pub id: Uuid,
    pub topic_id: Uuid,
    pub name: String,
}
