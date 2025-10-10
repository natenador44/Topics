use bson::serde_helpers::chrono_datetime_as_bson_datetime;
use chrono::{DateTime, Utc};
use engine::id::TopicId;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::fmt::Display;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde_as]
pub struct Topic {
    #[serde(rename = "_id")]
    pub id: TopicId,
    pub name: String,
    pub description: Option<String>,
    #[serde_as(as = "chrono_datetime_as_bson_datetime")]
    pub created: DateTime<Utc>,
    #[serde_as(as = "chrono_datetime_as_bson_datetime")]
    pub updated: Option<DateTime<Utc>>,
}
