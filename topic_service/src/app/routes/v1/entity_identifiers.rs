use axum::{
    Json,
    extract::{Path, Query},
    response::IntoResponse,
};
use serde::Deserialize;
use topic_core::v1::{EntityIdentifierId, TopicId};
use tracing::{Level, debug, instrument};

use crate::app::pagination::Pagination;

#[derive(Debug, Deserialize)]
pub struct EntityIdentifierSearch {}

#[derive(Debug, Deserialize)]
pub struct EntityIdentifierRequest {}

#[instrument(level=Level::DEBUG)]
pub async fn search(
    Path(topic_id): Path<TopicId>,
    Query(pagination): Query<Pagination>,
    Query(search_criteria): Query<EntityIdentifierSearch>,
) -> impl IntoResponse {
}

#[instrument(level=Level::DEBUG)]
pub async fn get(
    Path((topic_id, entity_identifier_id)): Path<(TopicId, EntityIdentifierId)>,
) -> impl IntoResponse {
}

#[instrument(level=Level::DEBUG)]
pub async fn create(
    Path(topic_id): Path<TopicId>,
    Json(new_entity_identifier): Json<EntityIdentifierRequest>,
) -> impl IntoResponse {
}

#[instrument(level=Level::DEBUG)]
pub async fn update(
    Path(topic_id): Path<TopicId>,
    Path(entity_identifier_id): Path<EntityIdentifierId>,
    Json(entity_identifier): Json<EntityIdentifierRequest>,
) -> impl IntoResponse {
}

#[instrument(level=Level::DEBUG)]
pub async fn delete(
    Path(topic_id): Path<TopicId>,
    Path(entity_identifier_id): Path<EntityIdentifierId>,
) -> impl IntoResponse {
}
