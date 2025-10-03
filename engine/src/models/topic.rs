use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Eq, Copy, Clone)]
#[repr(transparent)]
#[serde(transparent)]
#[schema(as = uuid::Uuid)]
pub struct TopicId(pub Uuid);
impl TopicId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Display for TopicId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, Eq)]
pub struct Topic {
    pub id: TopicId,
    pub name: String,
    pub description: Option<String>,
}

impl PartialEq for Topic {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Topic {
    pub fn new(id: TopicId, name: String, description: Option<String>) -> Topic {
        Self {
            id,
            name,
            description,
        }
    }

    pub fn new_random_id<N, D>(name: N, description: D) -> Self
    where
        N: Into<String>,
        D: Into<String>,
    {
        Self::new(TopicId::new(), name.into(), Some(description.into()))
    }
}
