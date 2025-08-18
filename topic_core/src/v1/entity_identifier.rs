use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::TopicId;

pub type EntityIdentifierId = Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct EntityIdentifier {
    pub id: EntityIdentifierId,
    pub topic_id: TopicId,
    // some sort of expression to 'test' data
}
