use crate::models::IdentifierId;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::{Display, Formatter};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Eq, Copy, Clone)]
#[repr(transparent)]
#[serde(transparent)]
#[schema(as = uuid::Uuid)]
pub struct EntityId(Uuid);
impl EntityId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Display for EntityId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
/// A JSON payload that is part of a set, which in turn is a part of a topic.
/// Each topic can have multiple sets, and each set can have multiple entities.
pub struct Entity {
    pub id: EntityId,
    pub applied_identifiers: Vec<IdentifierId>, // TODO better way to store/identify these
    pub payload: Value,                         // this will be different later
}
