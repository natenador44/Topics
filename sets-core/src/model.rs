use chrono::{DateTime, Utc};
use optional_field::Field;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, PartialEq, Eq)]
pub struct Set<K> {
    #[serde(flatten)]
    pub key: K,
    pub name: String,
    pub description: Option<String>,
    pub created: DateTime<Utc>,
    pub updated: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub struct NewSet {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Clone)]
pub struct PatchSet {
    pub name: Option<String>,
    pub description: Field<String>,
}
