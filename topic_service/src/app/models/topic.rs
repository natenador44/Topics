use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

pub type TopicId = Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct Topic {
    pub id: Uuid,
    pub name: String,
    pub description: String,
}

impl Topic {
    pub fn new<N, D>(name: N, description: D) -> Self
    where
        N: Into<String>,
        D: Into<String>,
    {
        Self {
            id: Uuid::now_v7(),
            name: name.into(),
            description: description.into(),
        }
    }
}
