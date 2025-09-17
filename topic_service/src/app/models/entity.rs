use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::app::models::IdentifierId;

pub type EntityId = Uuid;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
/// A JSON payload that is part of a set, which in turn is a part of a topic.
/// Each topic can have multiple sets, and each set can have multiple entities.
pub struct Entity {
    pub id: Uuid,
    pub topic_id: Uuid,
    pub set_id: Uuid,
    pub applied_identifiers: Vec<Uuid>, // TODO better way to store/identify these
    pub payload: Value,                 // this will be different later
}
