use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type TopicId = Uuid;

#[derive(Serialize, Deserialize)]
pub struct Topic {
    id: TopicId,
    name: String,
    description: String,
}
