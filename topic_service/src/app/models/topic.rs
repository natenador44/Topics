use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

pub type TopicId = Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, Eq)]
pub struct Topic {
    pub id: Uuid,
    pub name: String,
    pub description: String,
}

impl PartialEq for Topic {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id  
    }
}

impl Topic {
    pub fn new<N, D>(id: TopicId, name: N, description: D) -> Topic
    where
        N: Into<String>,
        D: Into<String>,
    {
        Self {
            id, name: name.into(), description: description.into(),
        }
    }
    pub fn new_random_id<N, D>(name: N, description: D) -> Self
    where
        N: Into<String>,
        D: Into<String>,
    {
        Self::new(Uuid::now_v7(), name, description)
    }
}
