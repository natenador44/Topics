use std::ops::Deref;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;

use super::TopicId;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EntityId {
    #[schema(value_type=String)]
    #[serde(flatten)]
    inner: Uuid,
}

impl EntityId {
    pub fn new() -> Self {
        Self {
            inner: Uuid::new_v4(),
        }
    }
}

impl Deref for EntityId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Entity {
    pub id: EntityId,
    pub topic_id: TopicId,
    pub payload: Value,
}
