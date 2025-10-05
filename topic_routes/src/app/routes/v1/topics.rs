use engine::patch_field_schema;
use std::fmt::Debug;

use crate::app::routes::response::StreamingResponse;
use crate::app::services::ResourceOutcome;
use crate::{
    app::{services::Service, state::AppState},
    error::{ServiceError, TopicServiceError},
};
use axum::routing::patch;
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response, Result},
    routing::{delete, get, post},
};
use chrono::{DateTime, Utc};
use engine::models::{Topic, TopicId};
use engine::search_criteria::SearchFilter;
use engine::search_filters::TopicFilter;
use engine::{Engine, Pagination};
use optional_field::{Field, serde_optional_fields};
use serde::{Deserialize, Deserializer, Serialize};
use tracing::{info, instrument};
use utoipa::openapi::{Object, ObjectBuilder, RefOr, Schema};
use utoipa::{OpenApi, PartialSchema, ToSchema, schema};
use utoipa_axum::router::OpenApiRouter;

#[derive(OpenApi)]
#[openapi(paths(search_topics, get_topic, create_topic, delete_topic, patch_topic,))]
pub struct ApiDoc;

#[derive(Debug, Deserialize, ToSchema)]
pub struct TopicRequest {
    name: String,
    description: Option<String>,
}

#[serde_optional_fields]
#[derive(Debug, Deserialize, ToSchema)]
pub struct TopicPatchRequest {
    #[schema(schema_with = patch_field_schema)]
    name: Field<String>,
    #[schema(schema_with = patch_field_schema)]
    description: Field<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct TopicSearch {
    name: Option<String>,
    description: Option<String>,
}
const DEFAULT_TOPIC_SEARCH_PAGE_SIZE: u32 = 25;

const TOPIC_SEARCH_PATH: &str = "/";
const TOPIC_GET_PATH: &str = "/{topic_id}";
const TOPIC_CREATE_PATH: &str = "/";
const TOPIC_DELETE_PATH: &str = "/{topic_id}";
const TOPIC_PATCH_PATH: &str = "/{topic_id}";

pub fn routes<T>() -> OpenApiRouter<AppState<T>>
where
    T: Engine + 'static,
{
    OpenApiRouter::new()
        .route(TOPIC_SEARCH_PATH, get(search_topics))
        .route(TOPIC_GET_PATH, get(get_topic))
        .route(TOPIC_CREATE_PATH, post(create_topic))
        .route(TOPIC_DELETE_PATH, delete(delete_topic))
        .route(TOPIC_PATCH_PATH, patch(patch_topic))
}

#[derive(Debug, Serialize, ToSchema)]
struct TopicResponse {
    #[serde(skip)]
    status_code: StatusCode,
    id: TopicId,
    name: String,
    description: Option<String>,
    created: DateTime<Utc>,
    updated: Option<DateTime<Utc>>,
    sets_url: String,
    identifiers_url: String,
}

impl TopicResponse {
    fn ok(topic: Topic) -> Self {
        Self {
            status_code: StatusCode::OK,
            id: topic.id,
            name: topic.name,
            description: topic.description,
            created: topic.created,
            updated: topic.updated,
            sets_url: format!("/api/v1/topics/{}/sets", topic.id),
            identifiers_url: format!("/api/v1/topics/{}/identifiers", topic.id),
        }
    }

    fn created(topic: Topic) -> Self {
        Self {
            status_code: StatusCode::CREATED,
            id: topic.id,
            name: topic.name,
            description: topic.description,
            created: topic.created,
            updated: topic.updated,
            sets_url: format!("/api/v1/topics/{}/sets", topic.id),
            identifiers_url: format!("/api/v1/topics/{}/identifiers", topic.id),
        }
    }
}

impl IntoResponse for TopicResponse {
    fn into_response(self) -> Response {
        (self.status_code, Json(self)).into_response()
    }
}

/// Query and filter for topics. Can specify pagination (page and page_size) to limit results returned.
/// Can also specify `SearchCriteria` to further reduce results based on name, description, or more.
#[utoipa::path(
    get,
    path = TOPIC_SEARCH_PATH,
    responses(
        (status = OK, description = "At least one topic was found given the search criteria", body = Vec<TopicResponse>),
        (status = NO_CONTENT, description = "No topics were found for the given search criteria"),
    ),
    params(
        ("page" = Option<usize>, Query, description = "Select certain page of results. Defaults to 1"),
        ("page_size" = Option<usize>, Query, description = "Max number of results to return per page. Defaults to ..."),
        ("name" = Option<String>, Query, description = "Filter topics by name"),
        ("description" = Option<String>, Query, description = "Filter topics by description"),
    )
)]
// #[axum::debug_handler]
#[instrument(skip_all, ret, err(Debug), fields(
    req.page = pagination.page,
    req.page_size = pagination.page_size,
    req.filter.name = search.name,
    req.filter.desc = search.description
))]
pub async fn search_topics<T>(
    State(service): State<Service<T>>,
    Query(pagination): Query<Pagination>,
    Query(search): Query<TopicSearch>,
) -> Result<Response, ServiceError<TopicServiceError>>
where
    T: Engine + Debug,
{
    info!("searching for topics..");
    let mut search_criteria = TopicFilter::criteria(pagination, DEFAULT_TOPIC_SEARCH_PAGE_SIZE);

    if let Some(name) = search.name {
        search_criteria.add(TopicFilter::Name(name));
    }

    if let Some(description) = search.description {
        search_criteria.add(TopicFilter::Description(description));
    }

    let topics = service.topics.search(search_criteria).await?;

    if topics.is_empty() {
        Ok(StatusCode::NO_CONTENT.into_response())
    } else {
        Ok(StreamingResponse::new(topics.into_iter().map(TopicResponse::ok)).into_response())
    }
}

/// Get the topic associated with the given id.
// #[axum::debug_handler]
#[utoipa::path(
    get,
    path = TOPIC_GET_PATH,
    responses(
        (status = OK, description = "A topic was found that matched the given TopicId", body = Topic),
        (status = NOT_FOUND, description = "No topics with the given TopicId were found"),
    ),
    params(
        ("topic_id" = TopicId, Path, description = "The TopicId to find"),
    )
)]
#[instrument(skip(service), ret, err(Debug))]
pub async fn get_topic<T>(
    State(service): State<Service<T>>,
    Path(topic_id): Path<TopicId>,
) -> Result<Response, ServiceError<TopicServiceError>>
where
    T: Engine + Debug,
{
    let topic = service.topics.get(topic_id).await?;

    Ok(topic
        .map(|t| TopicResponse::ok(t).into_response())
        .unwrap_or_else(|| StatusCode::NOT_FOUND.into_response()))
}

/// Create a new Topic and return its ID
#[utoipa::path(
    post,
    path = TOPIC_CREATE_PATH,
    responses(
        (status = CREATED, description = "A topic was successfully created", body = TopicResponse),
    ),
    request_body = TopicRequest
)]
#[instrument(skip_all, ret, err(Debug), fields(req.name = topic.name, req.description = topic.description))]
async fn create_topic<T>(
    State(service): State<Service<T>>,
    Json(topic): Json<TopicRequest>,
) -> Result<TopicResponse, ServiceError<TopicServiceError>>
where
    T: Engine + Debug,
{
    let new_topic = service.topics.create(topic.name, topic.description).await?;
    Ok(TopicResponse::created(new_topic))
}

/// Delete the topic associated with the given id
#[utoipa::path(
    delete,
    path = TOPIC_DELETE_PATH,
    responses(
        (status = NO_CONTENT, description = "The topic was successfully deleted, or never existed"),
    ),
    params(
        ("topic_id" = TopicId, Path, description = "The ID of the topic to delete to delete")
    )
)]
#[instrument(skip(service), ret, err(Debug))]
pub async fn delete_topic<T>(
    State(service): State<Service<T>>,
    Path(topic_id): Path<TopicId>,
) -> Result<StatusCode, ServiceError<TopicServiceError>>
where
    T: Engine + Debug,
{
    match service.topics.delete(topic_id).await? {
        ResourceOutcome::Found => Ok(StatusCode::NO_CONTENT),
        ResourceOutcome::NotFound => Ok(StatusCode::NOT_FOUND),
    }
}

/// Update the topic associated with the given id using the given information.
#[utoipa::path(
    patch,
    path = TOPIC_PATCH_PATH,
    responses(
        (status = OK, description = "The topic was successfully patched", body = Topic),
        (status = NOT_FOUND, description = "The topic was not found so could not be updated"),
    ),
    params(
        ("topic_id" = TopicId, Path, description = "The TopicId to patch")
    ),
    request_body = TopicPatchRequest,
)]
#[instrument(skip(service), ret, err(Debug))]
pub async fn patch_topic<T>(
    State(service): State<Service<T>>,
    Path(topic_id): Path<TopicId>,
    Json(topic): Json<TopicPatchRequest>,
) -> Result<Response, ServiceError<TopicServiceError>>
where
    T: Engine + Debug,
{
    let updated_topic = service
        .topics
        .update(topic_id, topic.name, topic.description)
        .await?;

    Ok(updated_topic
        .map(|t| Json(t).into_response())
        .unwrap_or_else(|| StatusCode::NOT_FOUND.into_response()))
}

// TODO do we want a PATCH?
