use axum::{
    Json,
    extract::{Path, Query},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::Value;
use topic_core::v1::{Entity, EntityId, TopicId};
use tracing::{Level, debug, instrument};
use uuid::Uuid;

use crate::app::pagination::Pagination;

#[derive(Debug, Deserialize)]
pub struct EntityRequest {
    payload: Value,
}

#[derive(Debug, Deserialize)]
pub struct EntitySearch {
    name: String,
    description: String,
}

#[instrument(level=Level::DEBUG)]
pub async fn search(
    Path(topic_id): Path<TopicId>,
    Query(pagination): Query<Pagination>,
    Query(search): Query<EntitySearch>,
) -> Response {
    // if topic_id is not found, 404
    // if no results given the search critera, NO_CONTENT
    // if results found, 200 with body of Json<Vec<Entity>>

    StatusCode::OK.into_response()
}

#[instrument(level=Level::DEBUG)]
pub async fn get(Path((topic_id, entity_id)): Path<(TopicId, EntityId)>) -> impl IntoResponse {}

#[instrument(level=Level::DEBUG)]
pub async fn create(
    Path(topic_id): Path<TopicId>,
    Json(entity): Json<EntityRequest>,
) -> impl IntoResponse {
    Json(Uuid::new_v4())
}

#[instrument(level=Level::DEBUG)]
pub async fn update(
    Path((topic_id, entity_id)): Path<(TopicId, EntityId)>,
    Json(entity): Json<EntityRequest>,
) -> Response {
    Json(Entity {
        id: EntityId::new_v4(),
        topic_id,
        payload: entity.payload,
    })
    .into_response()
}

#[instrument(level=Level::DEBUG)]
pub async fn delete(Path((topic_id, entity_id)): Path<(TopicId, EntityId)>) -> StatusCode {
    StatusCode::NO_CONTENT
}
