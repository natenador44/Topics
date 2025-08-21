use axum::{
    Json,
    extract::Path,
    response::IntoResponse,
    routing::{delete, post, put},
};
use serde::Deserialize;
use tracing::{Level, instrument};
use utoipa_axum::router::OpenApiRouter;

use crate::app::models::{EntityId, EntityIdentifierId, TopicId, TopicSetId};

#[derive(Debug, Deserialize)]
pub struct EmptySetRequest {}

const CREATE_EMPTY_PATH: &str = "/";
const ADD_ENTITY_PATH: &str = "/{set_id}/entities/{entity_id}";
const ADD_ENTITY_IDENTIFIER_PATH: &str = "/{set_id}/identifiers/{entity_identifier_id}";
const DELETE_SET_PATH: &str = "/{set_id}";
const REMOVE_ENTITY_PATH: &str = "/{set_id}/entities/{entity_id}";
const REMOVE_ENTITY_IDENTIFIER_PATH: &str = "/{set_id}/identifiers/{entity_identifier_id}";

pub fn routes() -> OpenApiRouter {
    OpenApiRouter::new()
        .route(CREATE_EMPTY_PATH, post(create_empty))
        .route(ADD_ENTITY_PATH, put(add_entity_to_set))
        .route(
            ADD_ENTITY_IDENTIFIER_PATH,
            put(add_entity_identifier_to_set),
        )
        .route(DELETE_SET_PATH, delete(delete_set))
        .route(REMOVE_ENTITY_PATH, delete(delete_entity_in_set))
        .route(
            REMOVE_ENTITY_IDENTIFIER_PATH,
            delete(delete_entity_identifier_in_set),
        )
}

#[instrument(level=Level::DEBUG)]
async fn create_empty(
    Path(topic_id): Path<TopicId>,
    Json(empty_set_request): Json<EmptySetRequest>,
) -> impl IntoResponse {
}

#[instrument(level=Level::DEBUG)]
async fn add_entity_to_set(
    Path((topic_id, set_id, topic_entity_id)): Path<(TopicId, TopicSetId, EntityId)>,
) -> impl IntoResponse {
}

#[instrument(level=Level::DEBUG)]
async fn add_entity_identifier_to_set(
    Path((topic_id, set_id, topic_entity_identifier_id)): Path<(
        TopicId,
        TopicSetId,
        EntityIdentifierId,
    )>,
) -> impl IntoResponse {
}

#[instrument(level=Level::DEBUG)]
async fn delete_set(Path((topic_id, set_id)): Path<(TopicId, TopicSetId)>) -> impl IntoResponse {}

#[instrument(level=Level::DEBUG)]
async fn delete_entity_in_set(
    Path((topic_id, set_id, entity_id)): Path<(TopicId, TopicSetId, EntityId)>,
) -> impl IntoResponse {
}

#[instrument(level=Level::DEBUG)]
async fn delete_entity_identifier_in_set(
    Path((topic_id, set_id, entity_identifier_id)): Path<(TopicId, TopicSetId, EntityIdentifierId)>,
) -> impl IntoResponse {
}
