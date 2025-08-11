use axum::{
    Json,
    extract::{Path, Query},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::Value;
use topic_core::v1::{Entity, EntityId, TopicId};
use tracing::debug;
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

pub async fn search(
    Path(topic_id): Path<TopicId>,
    Query(pagination): Query<Pagination>,
    Query(search): Query<EntitySearch>,
) -> Response {
    debug!("entity search -> pagination: {pagination:?}, search: {search:?}");

    // if topic_id is not found, 404
    // if no results given the search critera, NO_CONTENT
    // if results found, 200 with body of Json<Vec<Entity>>

    StatusCode::OK.into_response()
}

pub async fn get(Path((topic_id, entity_id)): Path<(TopicId, EntityId)>) -> impl IntoResponse {
    debug!("entity get -> topic_id: {topic_id}, entity id: {entity_id}");
}

pub async fn create(
    Path(topic_id): Path<TopicId>,
    Json(entity): Json<EntityRequest>,
) -> impl IntoResponse {
    debug!("entity create -> topic_id: {topic_id}, entity: {entity:?}");
    Json(Uuid::new_v4())
}

pub async fn update(
    Path((topic_id, entity_id)): Path<(TopicId, EntityId)>,
    Json(entity): Json<EntityRequest>,
) -> Response {
    debug!("entity update -> topic_id: {topic_id}, entity_id: {entity_id}, entity: {entity:?}");

    Json(Entity {
        id: EntityId::new_v4(),
        topic_id,
        payload: entity.payload,
    })
    .into_response()
}

pub async fn delete(Path((topic_id, entity_id)): Path<(TopicId, EntityId)>) -> StatusCode {
    debug!("entity delete -> topic_id: {topic_id}, entity_id: {entity_id}");
    StatusCode::NO_CONTENT
}
