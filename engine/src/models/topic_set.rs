use crate::models::TopicId;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Eq, Copy, Clone)]
#[repr(transparent)]
#[serde(transparent)]
#[schema(as = uuid::Uuid)]
pub struct SetId(pub Uuid);
impl SetId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Display for SetId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Set {
    pub id: SetId,
    pub topic_id: TopicId,
    pub name: String,
    pub description: Option<String>,
    pub created: DateTime<Utc>,
    pub updated: Option<DateTime<Utc>>,
}
