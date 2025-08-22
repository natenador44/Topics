use axum::{
    Json,
    extract::{Path, Query},
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use serde::Deserialize;
use tracing::{Level, instrument};
use utoipa::{OpenApi, ToSchema};
use utoipa_axum::router::OpenApiRouter;

use crate::app::{
    models::{EntityId, EntityIdentifier, EntityIdentifierId, TopicId},
    pagination::Pagination,
};

#[derive(OpenApi)]
#[openapi(paths(
    search_identifiers,
    get_identifier,
    create_identifier,
    delete_identifier,
    update_identifier,
))]
pub struct ApiDoc;

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
#[utoipa::path(
    get,
    path = ENTITY_IDENTIFIER_SEARCH_PATH,
    responses(
        (status = OK, description = "At least one entity identifier was found given the search criteria", body = Vec<EntityIdentifier>),
        (status = NO_CONTENT, description = "No entity identifiers were found for the given search criteria"),
        (status = NOT_FOUND, description = "The topic id does not exist")
    ),
    params(
        ("topic_id" = Uuid, Path, description = "The id of the topic associated with the entity identifier"),
        ("page" = Option<usize>, Query, description = "Select certain page of results"),
        ("page_size" = Option<usize>, Query, description = "Max number of results to return per page. Defaults to ..."),
        ("name" = Option<String>, Query, description = "Filter entity identifiers by name"),
        ("description" = Option<String>, Query, description = "Filter entity identifiers by description"),
    )
)]
async fn search_identifiers(
    Path(topic_id): Path<TopicId>,
    Query(pagination): Query<Pagination>,
    Query(search_criteria): Query<EntityIdentifierSearch>,
) -> impl IntoResponse {
}

#[instrument(level=Level::DEBUG)]
#[utoipa::path(
    get,
    path = ENTITY_IDENTIFIER_GET_PATH,
    responses(
        (status = OK, description = "An entity identifier was found that matched the given id", body = EntityIdentifier),
        (status = NOT_FOUND, description = "The topic id or entity identifier id does not exist"),
    ),
    params(
        ("topic_id" = Uuid, Path, description = "The id of the topic associated with the entity identifier"),
        ("entity_identifier_id" = Uuid, Path, description = "The id of the entity identifier to find"),
    )
)]
async fn get_identifier(
    Path((topic_id, entity_identifier_id)): Path<(TopicId, EntityIdentifierId)>,
) -> impl IntoResponse {
}

#[instrument(level=Level::DEBUG)]
#[utoipa::path(
    post,
    path = ENTITY_IDENTIFIER_CREATE_PATH,
    responses(
        (status = CREATED, description = "An entity identifier was successfully created", body = Uuid),
        (status = NOT_FOUND, description = "The topic id does not exist")
    ),
    params(
        ("topic_id" = Uuid, Path, description = "The id of the topic associated with the entity identifier"),
    ),
    request_body = EntityIdentifierRequest
)]
async fn create_identifier(
    Path(topic_id): Path<TopicId>,
    Json(new_entity_identifier): Json<EntityIdentifierRequest>,
) -> impl IntoResponse {
}

#[instrument(level=Level::DEBUG)]
#[utoipa::path(
    put,
    path = ENTITY_IDENTIFIER_UPDATE_PATH,
    responses(
        (status = OK, description = "The entity identifier was successfully updated", body = EntityIdentifier),
        (status = NOT_FOUND, description = "The topic id or entity identifier id was not found so could not be updated"),
    ),
    params(
        ("topic_id" = Uuid, Path, description = "The id of the topic associated with the entity identifier"),
        ("entity_identifier_id" = Uuid, Path, description = "The id of the entity identifier to update")
    ),
    request_body = EntityIdentifierRequest,
)]
async fn update_identifier(
    Path(topic_id): Path<TopicId>,
    Path(entity_identifier_id): Path<EntityIdentifierId>,
    Json(entity_identifier): Json<EntityIdentifierRequest>,
) -> impl IntoResponse {
}

#[instrument(level=Level::DEBUG)]
#[utoipa::path(
    delete,
    path = ENTITY_IDENTIFIER_DELETE_PATH,
    responses(
        (status = NO_CONTENT, description = "The entity identifier was successfully deleted, or never existed"),
        (status = NOT_FOUND, description = "The topic id does not exist"),
    ),
    params(
        ("topic_id" = Uuid, Path, description = "The id of the topic associated with the entity identifier"),
        ("entity_identifier_id" = Uuid, Path, description = "The id of the entity identifier to delete")
    )
)]
async fn delete_identifier(
    Path(topic_id): Path<TopicId>,
    Path(entity_identifier_id): Path<EntityIdentifierId>,
) -> impl IntoResponse {
}
