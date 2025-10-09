use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use engine::id::{SetId, TopicId};


#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Set {
    pub id: SetId,
    pub topic_id: TopicId,
    pub name: String,
    pub description: Option<String>,
    pub created: DateTime<Utc>,
    pub updated: Option<DateTime<Utc>>,
}
