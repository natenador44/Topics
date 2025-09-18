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
    models::{Identifier, IdentifierId, TopicId},
    pagination::Pagination,
    repository::Repository,
    state::AppState,
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
pub struct IdentifierSearch {}

#[derive(Debug, Deserialize, ToSchema)]
pub struct IdentifierRequest {}

const IDENTIFIER_SEARCH_PATH: &str = "/";
const IDENTIFIER_GET_PATH: &str = "/{identifier_id}";
const IDENTIFIER_CREATE_PATH: &str = "/";
const IDENTIFIER_DELETE_PATH: &str = "/{identifier_id}";
const IDENTIFIER_UPDATE_PATH: &str = "/{identifier_id}";

pub fn routes<T>() -> OpenApiRouter<AppState<T>>
where
    T: Repository + 'static,
{
    OpenApiRouter::new()
        .route(IDENTIFIER_SEARCH_PATH, get(search_identifiers))
        .route(IDENTIFIER_GET_PATH, get(get_identifier))
        .route(IDENTIFIER_CREATE_PATH, post(create_identifier))
        .route(IDENTIFIER_DELETE_PATH, delete(delete_identifier))
        .route(IDENTIFIER_UPDATE_PATH, put(update_identifier))
}

#[instrument(level=Level::DEBUG)]
#[utoipa::path(
    get,
    path = IDENTIFIER_SEARCH_PATH,
    responses(
        (status = OK, description = "At least one identifier was found given the search criteria", body = Vec<Identifier>),
        (status = NO_CONTENT, description = "No identifiers were found for the given search criteria"),
        (status = NOT_FOUND, description = "The topic id does not exist")
    ),
    params(
        ("topic_id" = TopicId, Path, description = "The id of the topic associated with the identifier"),
        ("page" = Option<usize>, Query, description = "Select certain page of results"),
        ("page_size" = Option<usize>, Query, description = "Max number of results to return per page. Defaults to ..."),
        ("name" = Option<String>, Query, description = "Filter identifiers by name"),
        ("description" = Option<String>, Query, description = "Filter identifiers by description"),
    )
)]
async fn search_identifiers(
    Path(topic_id): Path<TopicId>,
    Query(pagination): Query<Pagination>,
    Query(search_criteria): Query<IdentifierSearch>,
) -> impl IntoResponse {
}

#[instrument(level=Level::DEBUG)]
#[utoipa::path(
    get,
    path = IDENTIFIER_GET_PATH,
    responses(
        (status = OK, description = "An identifier was found that matched the given id", body = Identifier),
        (status = NOT_FOUND, description = "The topic id or identifier id does not exist"),
    ),
    params(
        ("topic_id" = TopicId, Path, description = "The id of the topic associated with the identifier"),
        ("identifier_id" = IdentifierId, Path, description = "The id of the identifier to find"),
    )
)]
async fn get_identifier(
    Path((topic_id, identifier_id)): Path<(TopicId, IdentifierId)>,
) -> impl IntoResponse {
}

#[instrument(level=Level::DEBUG)]
#[utoipa::path(
    post,
    path = IDENTIFIER_CREATE_PATH,
    responses(
        (status = CREATED, description = "An identifier was successfully created", body = IdentifierId),
        (status = NOT_FOUND, description = "The topic id does not exist")
    ),
    params(
        ("topic_id" = TopicId, Path, description = "The id of the topic associated with the identifier"),
    ),
    request_body = IdentifierRequest
)]
async fn create_identifier(
    Path(topic_id): Path<TopicId>,
    Json(new_identifier): Json<IdentifierRequest>,
) -> impl IntoResponse {
}

#[instrument(level=Level::DEBUG)]
#[utoipa::path(
    put,
    path = IDENTIFIER_UPDATE_PATH,
    responses(
        (status = OK, description = "The identifier was successfully updated", body = Identifier),
        (status = NOT_FOUND, description = "The topic id or identifier id was not found so could not be updated"),
    ),
    params(
        ("topic_id" = TopicId, Path, description = "The id of the topic associated with the identifier"),
        ("identifier_id" = IdentifierId, Path, description = "The id of the identifier to update")
    ),
    request_body = IdentifierRequest,
)]
async fn update_identifier(
    Path(topic_id): Path<TopicId>,
    Path(identifier_id): Path<IdentifierId>,
    Json(identifier): Json<IdentifierRequest>,
) -> impl IntoResponse {
}

#[instrument(level=Level::DEBUG)]
#[utoipa::path(
    delete,
    path = IDENTIFIER_DELETE_PATH,
    responses(
        (status = NO_CONTENT, description = "The identifier was successfully deleted, or never existed"),
        (status = NOT_FOUND, description = "The topic id does not exist"),
    ),
    params(
        ("topic_id" = TopicId, Path, description = "The id of the topic associated with the identifier"),
        ("identifier_id" = IdentifierId, Path, description = "The id of the identifier to delete")
    )
)]
async fn delete_identifier(
    Path(topic_id): Path<TopicId>,
    Path(identifier_id): Path<IdentifierId>,
) -> impl IntoResponse {
}
