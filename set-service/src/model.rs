use chrono::{DateTime, Utc};
use engine::id::{SetId, TopicId};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Set {
    pub id: SetId,
    pub topic_id: TopicId,
    pub name: String,
    pub description: Option<String>,
    pub created: DateTime<Utc>,
    pub updated: Option<DateTime<Utc>>,
}
