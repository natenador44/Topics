use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

pub type EntityIdentifierId = Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EntityIdentifier {
    pub id: Uuid,
    pub topic_id: Uuid,
    // some sort of expression to 'test' data
}
