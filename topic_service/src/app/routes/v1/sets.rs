use axum::{
    Json,
    extract::{Path, Query, State},
    http::{StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fmt::Debug;
use error_stack::{IntoReport, ResultExt};
use tracing::{Level, instrument};
use utoipa::{OpenApi, ToSchema};
use utoipa_axum::router::OpenApiRouter;

use crate::{
    app::{
        models::{Entity, EntityId, TopicId, TopicSetId},
        pagination::Pagination,
        repository::Repository,
        services::Service,
        state::AppState,
    },
    error::{AppResult, ServiceError, SetServiceError},
};
use crate::app::models::TopicSet;

#[derive(OpenApi)]
#[openapi(paths(
    create_set,
    search_entities_in_set,
    get_entity_in_set,
    add_entity_to_set,
    delete_set,
    delete_entity_in_set,
))]
pub struct ApiDoc;

#[derive(Debug, Deserialize, ToSchema)]
pub struct SetRequest {
    name: String,
    entities: Vec<Value>,
}

const CREATE_SET_PATH: &str = "/";
const ADD_ENTITY_PATH: &str = "/{set_id}/entities";
const SEARCH_ENTITIES_PATH: &str = "/{set_id}/entities";
const GET_ENTITY_PATH: &str = "/{set_id}/entities/{entity_id}";
const DELETE_SET_PATH: &str = "/{set_id}";
const REMOVE_ENTITY_PATH: &str = "/{set_id}/entities/{entity_id}";

pub fn routes<T>() -> OpenApiRouter<AppState<T>>
where
    T: Repository + 'static,
{
    OpenApiRouter::new()
        .route(CREATE_SET_PATH, post(create_set))
        .route(SEARCH_ENTITIES_PATH, get(search_entities_in_set))
        .route(GET_ENTITY_PATH, get(get_entity_in_set))
        .route(ADD_ENTITY_PATH, put(add_entity_to_set))
        .route(DELETE_SET_PATH, delete(delete_set))
        .route(REMOVE_ENTITY_PATH, delete(delete_entity_in_set))
}

#[instrument(level=Level::DEBUG)]
#[utoipa::path(
    post,
    path = CREATE_SET_PATH,
    responses(
        (status = CREATED, description = "A set was successfully created", body = Uuid),
        (status = NOT_FOUND, description = "The topic id does not exist"),
    ),
    params(
        ("topic_id" = Uuid, Path, description = "The topic associated with the new set")
    ),
    request_body = SetRequest,
)]
async fn create_set<T>(
    State(service): State<Service<T>>,
    Path(topic_id): Path<TopicId>,
    Json(set_request): Json<SetRequest>,
) -> Result<(StatusCode, Json<TopicSet>), ServiceError<SetServiceError>>
where
    T: Repository + Debug,
{
    let new_set = service
        .sets
        .create(topic_id, set_request.name, set_request.entities).await?;


    Ok((StatusCode::CREATED, Json(new_set)))
}

#[derive(Deserialize, ToSchema, Debug)]
enum EntitySearch {
    /// Search through all entities, through all keys and values, for a fuzzy match to the given String
    FuzzySearch(String),
    // something like a list of identifiers
}

#[derive(Serialize, ToSchema, Debug)]
struct EntityResponse {
    entity: Entity,
    identities_urls: Vec<String>,
    set_url: String,
    topic_url: String,
}

#[utoipa::path(
    get,
    path = SEARCH_ENTITIES_PATH,
    responses(
        (status = OK, description = "Entities were found that matched the search criteria", body = Vec<EntityResponse>),
        (status = NO_CONTENT, description = "No entities were found that matched the search criteria"),
        (status = NOT_FOUND, description = "The topic id or the set id does not exist")
    ),
    params(
        ("topic_id" = Uuid, Path, description = "The topic associated with the set"),
        ("set_id" = Uuid, Path, description = "The set to search through")
    ),
)]
// #[axum::debug_handler]
async fn search_entities_in_set<T>(
    State(service): State<Service<T>>,
    Path((topic_id, set_id)): Path<(TopicId, TopicSetId)>,
    Query(search): Query<EntitySearch>,
    Query(Pagination { page, page_size }): Query<Pagination>,
) -> Result<Response, ServiceError<SetServiceError>>
where
    T: Repository + Debug,
{
    todo!()
}

#[utoipa::path(
    get,
    path = GET_ENTITY_PATH,
    responses(
        (status = OK, description = "The entity in the given topic set was found", body = EntityResponse),
        (status = NOT_FOUND, description = "The topic id, set id, or entity id does not exist")
    ),
    params(
        ("topic_id" = Uuid, Path, description = "The topic associated with the set the entity belongs to"),
        ("set_id" = Uuid, Path, description = "The set the entity belongs to"),
        ("entity_id" = Uuid, Path, description = "The entity to get")
    ),
)]
async fn get_entity_in_set<T>(
    State(service): State<Service<T>>,
    Path((topic_id, set_id, entity_id)): Path<(TopicId, TopicSetId, EntityId)>,
) -> Result<Response, ServiceError<SetServiceError>>
where
    T: Repository + Debug,
{
    todo!()
}

#[instrument(level=Level::DEBUG)]
#[utoipa::path(
    put,
    path = ADD_ENTITY_PATH,
    responses(
        (status = CREATED, description = "The entity was created and added to the set. Returns the ID of the new entity", body = Uuid),
        (status = NOT_FOUND, description = "The topic id or the set id does not exist")
    ),
    params(
        ("topic_id" = Uuid, Path, description = "The topic associated with the new set"),
        ("set_id" = Uuid, Path, description = "The set to add the new entity to")
    ),
    request_body = SetRequest,
)]
async fn add_entity_to_set<T>(
    State(service): State<Service<T>>,
    Path((topic_id, set_id)): Path<(TopicId, TopicSetId)>,
) -> Result<Response, ServiceError<SetServiceError>>
where
    T: Repository + Debug,
{
    todo!()
}

#[instrument(level=Level::DEBUG)]
#[utoipa::path(
    delete,
    path = DELETE_SET_PATH,
    responses(
        (status = NO_CONTENT, description = "The set was deleted or never existed"),
        (status = NOT_FOUND, description = "The topic id does not exist"),
    ),
    params(
        ("topic_id" = Uuid, Path, description = "The topic associated with the set"),
        ("set_id" = Uuid, Path, description = "The set to delete")
    ),
)]
async fn delete_set<T>(
    State(service): State<Service<T>>,
    Path((topic_id, set_id)): Path<(TopicId, TopicSetId)>,
) -> Result<StatusCode, ServiceError<SetServiceError>>
where
    T: Repository + Debug,
{
    todo!()
}

#[instrument(level=Level::DEBUG)]
#[utoipa::path(
    delete,
    path = REMOVE_ENTITY_PATH,
    responses(
        (status = NO_CONTENT, description = "The entity was deleted or never existed"),
        (status = NOT_FOUND, description = "The topic id or set id does not exist"),
    ),
    params(
        ("topic_id" = Uuid, Path, description = "The topic associated with the new set"),
        ("set_id" = Uuid, Path, description = "The set to add the new entity to"),
        ("entity_id" = Uuid, Path, description = "The id associated with the entity to remove")
    ),
)]
async fn delete_entity_in_set<T>(
    State(service): State<Service<T>>,
    Path((topic_id, set_id, entity_id)): Path<(TopicId, TopicSetId, EntityId)>,
) -> Result<StatusCode, ServiceError<SetServiceError>>
where
    T: Repository + Debug,
{
    todo!()
}
