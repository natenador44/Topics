use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::v1::TopicId;

pub type EntityId = Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct Entity {
    pub id: EntityId,
    pub topic_id: TopicId,
    pub payload: Value,
}
