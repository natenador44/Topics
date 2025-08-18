use axum::{Json, extract::Path, response::IntoResponse};
use serde::Deserialize;
use topic_core::v1::{EntityId, EntityIdentifierId, TopicId, TopicSetId};
use tracing::{Level, instrument};

#[derive(Debug, Deserialize)]
pub struct EmptySetRequest {}

#[instrument(level=Level::DEBUG)]
pub async fn create_empty(
    Path(topic_id): Path<TopicId>,
    Json(empty_set_request): Json<EmptySetRequest>,
) -> impl IntoResponse {
}

#[instrument(level=Level::DEBUG)]
pub async fn add_entity_to_set(
    Path((topic_id, set_id, topic_entity_id)): Path<(TopicId, TopicSetId, EntityId)>,
) -> impl IntoResponse {
}

#[instrument(level=Level::DEBUG)]
pub async fn add_entity_identifier_to_set(
    Path((topic_id, set_id, topic_entity_identifier_id)): Path<(
        TopicId,
        TopicSetId,
        EntityIdentifierId,
    )>,
) -> impl IntoResponse {
}

#[instrument(level=Level::DEBUG)]
pub async fn delete_set(
    Path((topic_id, set_id)): Path<(TopicId, TopicSetId)>,
) -> impl IntoResponse {
}

#[instrument(level=Level::DEBUG)]
pub async fn delete_entity_in_set(
    Path((topic_id, set_id, entity_id)): Path<(TopicId, TopicSetId, EntityId)>,
) -> impl IntoResponse {
}

#[instrument(level=Level::DEBUG)]
pub async fn delete_entity_identifier_in_set(
    Path((topic_id, set_id, entity_identifier_id)): Path<(TopicId, TopicSetId, EntityIdentifierId)>,
) -> impl IntoResponse {
}
