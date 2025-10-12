use chrono::{DateTime, Utc};
use optional_field::Field;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub struct NewTopic {
    pub name: String,
    pub description: Option<String>,
}

impl NewTopic {
    pub fn new(name: String, description: Option<String>) -> Self {
        Self { name, description }
    }
}

pub struct PatchTopic {
    pub name: Option<String>,
    pub description: Field<String>,
}

impl PatchTopic {
    pub fn new(name: Option<String>, description: Field<String>) -> Self {
        Self { name, description }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Topic<T> {
    pub id: T,
    pub name: String,
    pub description: Option<String>,
    pub created: DateTime<Utc>,
    pub updated: Option<DateTime<Utc>>,
}
