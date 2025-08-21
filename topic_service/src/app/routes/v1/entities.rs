use axum::{
    Json,
    extract::{Path, Query},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
};
use serde::Deserialize;
use serde_json::Value;
use tracing::{Level, instrument};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use uuid::Uuid;

use crate::app::{
    models::{Entity, EntityId, TopicId},
    pagination::Pagination,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct EntityRequest {
    payload: Value,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct EntitySearch {
    name: String,
    description: String,
}

const ENTITY_SEARCH_PATH: &str = "/";
const ENTITY_GET_PATH: &str = "/{entity_id}";
const ENTITY_CREATE_PATH: &str = "/";
const ENTITY_DELETE_PATH: &str = "/{entity_id}";
const ENTITY_UPDATE_PATH: &str = "/{entity_id}";

pub fn routes() -> OpenApiRouter {
    OpenApiRouter::new()
        .route(ENTITY_SEARCH_PATH, get(search_entities))
        .route(ENTITY_GET_PATH, get(get_entity))
        .route(ENTITY_CREATE_PATH, post(create_entity))
        .route(ENTITY_DELETE_PATH, delete(delete_entity))
        .route(ENTITY_UPDATE_PATH, put(update_entity))
}

#[instrument(level=Level::DEBUG)]
pub async fn search_entities(
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
pub async fn get_entity(
    Path((topic_id, entity_id)): Path<(TopicId, EntityId)>,
) -> impl IntoResponse {
}

#[instrument(level=Level::DEBUG)]
pub async fn create_entity(
    Path(topic_id): Path<TopicId>,
    Json(entity): Json<EntityRequest>,
) -> impl IntoResponse {
    Json(Uuid::new_v4())
}

#[instrument(level=Level::DEBUG)]
pub async fn update_entity(
    Path((topic_id, entity_id)): Path<(TopicId, EntityId)>,
    Json(entity): Json<EntityRequest>,
) -> Response {
    Json(Entity {
        id: EntityId::new(),
        topic_id,
        payload: entity.payload,
    })
    .into_response()
}

#[instrument(level=Level::DEBUG)]
pub async fn delete_entity(Path((topic_id, entity_id)): Path<(TopicId, EntityId)>) -> StatusCode {
    StatusCode::NO_CONTENT
}
