use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use super::TopicId;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
pub struct EntityIdentifierId {
    #[schema(value_type=String)]
    inner: Uuid,
}

impl EntityIdentifierId {
    pub fn new() -> Self {
        Self {
            inner: Uuid::new_v4(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EntityIdentifier {
    pub id: EntityIdentifierId,
    pub topic_id: TopicId,
    // some sort of expression to 'test' data
}
