use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
};
use error_stack::IntoReport;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::fmt::Debug;
use tracing::{Level, instrument};
use utoipa::{OpenApi, ToSchema};
use utoipa_axum::router::OpenApiRouter;

use crate::app::models::Set;
use crate::error::TopicServiceError;
use crate::{
    app::{
        models::{Entity, EntityId, SetId, TopicId},
        pagination::Pagination,
        repository::Repository,
        services::Service,
        state::AppState,
    },
    error::{ServiceError, SetServiceError},
};

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
    entities: Option<Vec<Value>>,
}

const CREATE_SET_PATH: &str = "/";
const GET_SET_PATH: &str = "/{set_id}";
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
        .route(GET_SET_PATH, get(get_set))
        .route(SEARCH_ENTITIES_PATH, get(search_entities_in_set))
        .route(GET_ENTITY_PATH, get(get_entity_in_set))
        .route(ADD_ENTITY_PATH, put(add_entity_to_set))
        .route(DELETE_SET_PATH, delete(delete_set))
        .route(REMOVE_ENTITY_PATH, delete(delete_entity_in_set))
}

#[derive(Debug, ToSchema, Serialize)]
struct SetResponse {
    #[serde(skip)]
    status_code: StatusCode,
    id: SetId,
    topic_id: TopicId,
    name: String,
    entities_url: String,
}

impl SetResponse {
    fn ok(set: Set) -> Self {
        Self {
            status_code: StatusCode::OK,
            id: set.id,
            topic_id: set.topic_id,
            name: set.name,
            entities_url: format!("/api/v1/topics/{}/sets/{}/entities", set.topic_id, set.id),
        }
    }

    fn created(set: Set) -> Self {
        Self {
            status_code: StatusCode::CREATED,
            id: set.id,
            topic_id: set.topic_id,
            name: set.name,
            entities_url: format!("/api/v1/topics/{}/sets/{}/entities", set.topic_id, set.id),
        }
    }
}

impl IntoResponse for SetResponse {
    fn into_response(self) -> Response {
        (
            self.status_code,
            Json(json!({
                    "id": self.id,
                    "topic_id": self.topic_id,
                    "name": self.name,
                    "entities_url": self.entities_url,
                }
            )),
        )
            .into_response()
    }
}

#[utoipa::path(
    post,
    path = CREATE_SET_PATH,
    responses(
        (status = CREATED, description = "A set was successfully created", body = SetResponse),
        (status = NOT_FOUND, description = "The topic id does not exist"),
    ),
    params(
        ("topic_id" = TopicId, Path, description = "The topic ID associated with the new set")
    ),
    request_body = SetRequest,
)]
#[instrument(skip(service, set_request), ret, err(Debug), fields(
    req.name = set_request.name,
    req.entity_count = set_request.entities.as_ref().map_or(0, Vec::len)
))]
async fn create_set<T>(
    State(service): State<Service<T>>,
    Path(topic_id): Path<TopicId>,
    Json(set_request): Json<SetRequest>,
) -> Result<Response, ServiceError<SetServiceError>>
where
    T: Repository + Debug,
{
    let new_set = service
        .sets
        .create(topic_id, set_request.name, set_request.entities)
        .await?;

    match new_set {
        Some(new_set) => Ok(SetResponse::created(new_set).into_response()),
        // TODO find a way to have ? automatically return a 404 response if this is true
        None => Ok(StatusCode::NOT_FOUND.into_response()),
    }
}

#[utoipa::path(
    get,
    path = GET_SET_PATH,
    responses(
        (status = OK, description = "Set was found", body = Vec<Set>),
        (status = NOT_FOUND, description = "The topic id or the set id does not exist")
    ),
    params(
        ("topic_id" = TopicId, Path, description = "The topic associated with the set"),
        ("set_id" = SetId, Path, description = "The set to get")
    ),
)]
// #[axum::debug_handler]
#[instrument(skip(service), ret, err(Debug))]
async fn get_set<T>(
    State(service): State<Service<T>>,
    Path((topic_id, set_id)): Path<(TopicId, SetId)>,
) -> Result<Response, ServiceError<SetServiceError>>
where
    T: Repository + Debug,
{
    let set = service.sets.get(topic_id, set_id).await?;

    Ok(set
        .map(|s| SetResponse::ok(s).into_response())
        .unwrap_or_else(|| StatusCode::NOT_FOUND.into_response()))
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
        ("topic_id" = TopicId, Path, description = "The topic associated with the set"),
        ("set_id" = SetId, Path, description = "The set to search through")
    ),
)]
// #[axum::debug_handler]
async fn search_entities_in_set<T>(
    State(service): State<Service<T>>,
    Path((topic_id, set_id)): Path<(TopicId, SetId)>,
    // Query(search): Query<EntitySearch>,
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
        ("topic_id" = TopicId, Path, description = "The topic associated with the set the entity belongs to"),
        ("set_id" = SetId, Path, description = "The set the entity belongs to"),
        ("entity_id" = EntityId, Path, description = "The entity to get")
    ),
)]
async fn get_entity_in_set<T>(
    State(service): State<Service<T>>,
    Path((topic_id, set_id, entity_id)): Path<(TopicId, SetId, EntityId)>,
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
        (status = CREATED, description = "The entity was created and added to the set. Returns the ID of the new entity", body = EntityId),
        (status = NOT_FOUND, description = "The topic id or the set id does not exist")
    ),
    params(
        ("topic_id" = TopicId, Path, description = "The topic associated with the new set"),
        ("set_id" = SetId, Path, description = "The set to add the new entity to")
    ),
    request_body = SetRequest,
)]
async fn add_entity_to_set<T>(
    State(service): State<Service<T>>,
    Path((topic_id, set_id)): Path<(TopicId, SetId)>,
) -> Result<Response, ServiceError<SetServiceError>>
where
    T: Repository + Debug,
{
    todo!()
}

#[utoipa::path(
    delete,
    path = DELETE_SET_PATH,
    responses(
        (status = NO_CONTENT, description = "The set was deleted or never existed"),
        (status = NOT_FOUND, description = "The topic id does not exist"),
    ),
    params(
        ("topic_id" = TopicId, Path, description = "The topic associated with the set"),
        ("set_id" = SetId, Path, description = "The set to delete")
    ),
)]
#[instrument(skip(service), ret, err(Debug))]
async fn delete_set<T>(
    State(service): State<Service<T>>,
    Path((topic_id, set_id)): Path<(TopicId, SetId)>,
) -> Result<StatusCode, ServiceError<SetServiceError>>
where
    T: Repository + Debug,
{
    service.sets.delete(topic_id, set_id).await?;
    Ok(StatusCode::NO_CONTENT)
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
        ("topic_id" = TopicId, Path, description = "The topic associated with the new set"),
        ("set_id" = SetId, Path, description = "The set to add the new entity to"),
        ("entity_id" = EntityId, Path, description = "The id associated with the entity to remove")
    ),
)]
async fn delete_entity_in_set<T>(
    State(service): State<Service<T>>,
    Path((topic_id, set_id, entity_id)): Path<(TopicId, SetId, EntityId)>,
) -> Result<StatusCode, ServiceError<SetServiceError>>
where
    T: Repository + Debug,
{
    todo!()
}
