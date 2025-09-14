use axum::{
    Json,
    extract::Path,
    response::IntoResponse,
    routing::{delete, post, put},
};
use serde::Deserialize;
use serde_json::Value;
use tracing::{Level, instrument};
use utoipa::{OpenApi, ToSchema};
use utoipa_axum::router::OpenApiRouter;

use crate::app::{
    models::{EntityId, TopicId, TopicSetId},
    repository::Repository,
    state::AppState,
};

#[derive(OpenApi)]
#[openapi(paths(create_set, add_entity_to_set, delete_set, delete_entity_in_set,))]
pub struct ApiDoc;

#[derive(Debug, Deserialize, ToSchema)]
pub struct SetRequest {
    name: String,
    entities: Vec<Value>,
}

const CREATE_SET_PATH: &str = "/";
const ADD_ENTITY_PATH: &str = "/{set_id}/entities";
const DELETE_SET_PATH: &str = "/{set_id}";
const REMOVE_ENTITY_PATH: &str = "/{set_id}/entities/{entity_id}";

pub fn routes<T>() -> OpenApiRouter<AppState<T>>
where
    T: Repository + 'static,
{
    OpenApiRouter::new()
        .route(CREATE_SET_PATH, post(create_set))
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
async fn create_set(
    Path(topic_id): Path<TopicId>,
    Json(set_request): Json<SetRequest>,
) -> impl IntoResponse {
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
async fn add_entity_to_set(
    Path((topic_id, set_id)): Path<(TopicId, TopicSetId)>,
) -> impl IntoResponse {
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
        ("topic_id" = Uuid, Path, description = "The topic associated with the new set"),
        ("set_id" = Uuid, Path, description = "The set to add the new entity to")
    ),
)]
async fn delete_set(Path((topic_id, set_id)): Path<(TopicId, TopicSetId)>) -> impl IntoResponse {}

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
async fn delete_entity_in_set(
    Path((topic_id, set_id, entity_id)): Path<(TopicId, TopicSetId, EntityId)>,
) -> impl IntoResponse {
}
