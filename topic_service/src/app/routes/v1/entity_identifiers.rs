use axum::{
    Json,
    extract::{Path, Query},
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use serde::Deserialize;
use tracing::{Level, instrument};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;

use crate::app::{
    models::{EntityIdentifierId, TopicId},
    pagination::Pagination,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct EntityIdentifierSearch {}

#[derive(Debug, Deserialize, ToSchema)]
pub struct EntityIdentifierRequest {}

const ENTITY_IDENTIFIER_SEARCH_PATH: &str = "/";
const ENTITY_IDENTIFIER_GET_PATH: &str = "/{entity_identifier_id}";
const ENTITY_IDENTIFIER_CREATE_PATH: &str = "/";
const ENTITY_IDENTIFIER_DELETE_PATH: &str = "/{entity_identifier_id}";
const ENTITY_IDENTIFIER_UPDATE_PATH: &str = "/{entity_identifier_id}";

pub fn routes() -> OpenApiRouter {
    OpenApiRouter::new()
        .route(ENTITY_IDENTIFIER_SEARCH_PATH, get(search_identifiers))
        .route(ENTITY_IDENTIFIER_GET_PATH, get(get_identifier))
        .route(ENTITY_IDENTIFIER_CREATE_PATH, post(create_identifier))
        .route(ENTITY_IDENTIFIER_DELETE_PATH, delete(delete_identifier))
        .route(ENTITY_IDENTIFIER_UPDATE_PATH, put(update_identifier))
}

#[instrument(level=Level::DEBUG)]
async fn search_identifiers(
    Path(topic_id): Path<TopicId>,
    Query(pagination): Query<Pagination>,
    Query(search_criteria): Query<EntityIdentifierSearch>,
) -> impl IntoResponse {
}

#[instrument(level=Level::DEBUG)]
async fn get_identifier(
    Path((topic_id, entity_identifier_id)): Path<(TopicId, EntityIdentifierId)>,
) -> impl IntoResponse {
}

#[instrument(level=Level::DEBUG)]
async fn create_identifier(
    Path(topic_id): Path<TopicId>,
    Json(new_entity_identifier): Json<EntityIdentifierRequest>,
) -> impl IntoResponse {
}

#[instrument(level=Level::DEBUG)]
async fn update_identifier(
    Path(topic_id): Path<TopicId>,
    Path(entity_identifier_id): Path<EntityIdentifierId>,
    Json(entity_identifier): Json<EntityIdentifierRequest>,
) -> impl IntoResponse {
}

#[instrument(level=Level::DEBUG)]
async fn delete_identifier(
    Path(topic_id): Path<TopicId>,
    Path(entity_identifier_id): Path<EntityIdentifierId>,
) -> impl IntoResponse {
}
