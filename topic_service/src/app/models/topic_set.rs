use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use crate::app::models::TopicId;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Eq, Copy, Clone)]
#[repr(transparent)]
#[serde(transparent)]
#[schema(as = uuid::Uuid)]
pub struct TopicSetId(Uuid);
impl TopicSetId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Display for TopicSetId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TopicSet {
    pub id: TopicSetId,
    pub topic_id: TopicId,
    pub name: String,
}
