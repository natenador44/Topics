use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::app::models::Entity;

pub type TopicSetId = Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TopicSet {
    pub id: Uuid,
    pub name: String,
    pub entities: Vec<Entity>,
}
