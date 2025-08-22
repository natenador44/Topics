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
use utoipa::{OpenApi, ToSchema};
use utoipa_axum::router::OpenApiRouter;
use uuid::Uuid;

use crate::app::{
    models::{Entity, EntityId, TopicId},
    pagination::Pagination,
};

#[derive(OpenApi)]
#[openapi(paths(
    search_entities,
    get_entity,
    create_entity,
    delete_entity,
    update_entity
))]
pub struct ApiDoc;

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
#[utoipa::path(
    get,
    path = ENTITY_SEARCH_PATH,
    responses(
        (status = OK, description = "At least one entity was found given the search criteria", body = Vec<Entity>),
        (status = NO_CONTENT, description = "No entities were found for the given search criteria"),
    ),
    params(
        ("topic_id" = Uuid, Path, description = "The id of the topic associated with the entity"),
        ("page" = Option<usize>, Query, description = "Select certain page of results. Defaults to 1"),
        ("page_size" = Option<usize>, Query, description = "Max number of results to return per page. Defaults to ..."),
        ("name" = Option<String>, Query, description = "Filter entities by name"),
        ("description" = Option<String>, Query, description = "Filter entities by description"),
    )
)]
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
#[utoipa::path(
    get,
    path = ENTITY_GET_PATH,
    responses(
        (status = OK, description = "An entity was found that matched the given id", body = Entity),
        (status = NOT_FOUND, description = "No entities with the given id were found"),
    ),
    params(
        ("topic_id" = Uuid, Path, description = "The id of the topic associated with the entity"),
        ("entity_id" = Uuid, Path, description = "The id of the entity to find"),
    )
)]
pub async fn get_entity(
    Path((topic_id, entity_id)): Path<(TopicId, EntityId)>,
) -> impl IntoResponse {
}

#[instrument(level=Level::DEBUG)]
#[utoipa::path(
    post,
    path = ENTITY_CREATE_PATH,
    responses(
        (status = CREATED, description = "An entity was successfully created", body = Uuid),
    ),
    params(
        ("topic_id" = Uuid, Path, description = "The id of the topic associated with the entity"),
    ),
    request_body = EntityRequest
)]
pub async fn create_entity(
    Path(topic_id): Path<TopicId>,
    Json(entity): Json<EntityRequest>,
) -> impl IntoResponse {
    Json(Uuid::new_v4())
}

#[instrument(level=Level::DEBUG)]
#[utoipa::path(
    put,
    path = ENTITY_UPDATE_PATH,
    responses(
        (status = OK, description = "The entity was successfully updated", body = Entity),
        (status = NOT_FOUND, description = "The entity was not found so could not be updated"),
    ),
    params(
        ("topic_id" = Uuid, Path, description = "The id of the topic associated with the entity"),
        ("entity_id" = Uuid, Path, description = "The id of the entity to update")
    ),
    request_body = EntityRequest,
)]
pub async fn update_entity(
    Path((topic_id, entity_id)): Path<(TopicId, EntityId)>,
    Json(entity): Json<EntityRequest>,
) -> impl IntoResponse {
}

#[instrument(level=Level::DEBUG)]
#[utoipa::path(
    delete,
    path = ENTITY_DELETE_PATH,
    responses(
        (status = NO_CONTENT, description = "The entity was successfully deleted, or never existed"),
    ),
    params(
        ("topic_id" = Uuid, Path, description = "The id of the topic associated with the entity"),
        ("entity_id" = Uuid, Path, description = "The id of the entity to delete")
    )
)]
pub async fn delete_entity(Path((topic_id, entity_id)): Path<(TopicId, EntityId)>) -> StatusCode {
    StatusCode::NO_CONTENT
}
