
mod responses;
mod requests;
use engine::{patch_field_schema, Pagination};
use std::fmt::Debug;

use axum::routing::patch;
use axum::{Json, extract::{Path, Query, State}, http::StatusCode, response::{IntoResponse, Response, Result}, routing::{delete, get, post}, Router};
use chrono::{DateTime, Utc};
use optional_field::{Field, serde_optional_fields};
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};
use utoipa::OpenApi;
use utoipa::ToSchema;
use utoipa::schema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;
use engine::error::ServiceError;
use engine::id::TopicId;
use crate::error::TopicServiceError;
use crate::model::Topic;
use crate::service::TopicService;
use crate::state::TopicAppState;

const TOPIC_ROOT_PATH: &str = "/topics";

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = TOPIC_ROOT_PATH, api = TopicDocs),
    )
)]
struct ApiDoc;

#[derive(OpenApi)]
#[openapi(paths(list_topics, get_topic, create_topic, delete_topic, patch_topic,))]
struct TopicDocs;

#[derive(Debug, Deserialize, ToSchema)]
pub struct TopicRequest {
    name: String,
    description: Option<String>,
}

#[serde_optional_fields]
#[derive(Debug, Deserialize, ToSchema)]
pub struct TopicPatchRequest {
    /// The new name of the topic. Cannot be null. If set to null or not specified, no update will happen.
    name: Option<String>,
    /// The new description of the topic. Can be null. If specified as null, the description will update to null.
    /// If not specified, no update will happen.
    #[schema(schema_with = patch_field_schema)]
    description: Field<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct TopicSearch {
    name: Option<String>,
    description: Option<String>,
}
const DEFAULT_TOPIC_SEARCH_PAGE_SIZE: u32 = 25;

const TOPIC_LIST_PATH: &str = "/";
const TOPIC_GET_PATH: &str = "/{topic_id}";
const TOPIC_CREATE_PATH: &str = "/";
const TOPIC_DELETE_PATH: &str = "/{topic_id}";
const TOPIC_PATCH_PATH: &str = "/{topic_id}";

pub fn build(app_state: TopicAppState) -> Router {
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(routes(app_state))
        .split_for_parts();

    router.merge(SwaggerUi::new("/topics/swagger-ui").url("/topics/api-docs/openapi.json", api))
}

fn routes<S>(app_state: TopicAppState) -> OpenApiRouter<S> {
    OpenApiRouter::new()
        .nest(TOPIC_ROOT_PATH,
              OpenApiRouter::new()
                  .route(TOPIC_LIST_PATH, get(list_topics))
                  .route(TOPIC_GET_PATH, get(get_topic))
                  .route(TOPIC_CREATE_PATH, post(create_topic))
                  .route(TOPIC_DELETE_PATH, delete(delete_topic))
                  .route(TOPIC_PATCH_PATH, patch(patch_topic))
        )
        .with_state(app_state)
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

/// Get the topic associated with the given id.
// #[axum::debug_handler]
#[utoipa::path(
    get,
    path = TOPIC_LIST_PATH,
    responses(
        (status = OK, description = "Topics were found on the given page", body = Vec<Topic>),
        (status = NO_CONTENT, description = "No topics exist on the given page"),
    ),
    params(
        ("page" = u32, Query, description = "The offset page to start the listing with"),
        ("page_size" = u32, Query, description = "The max number of topics to return"),
    )
)]
#[instrument(skip(service), err(Debug))]
pub async fn list_topics(
    State(service): State<TopicService>,
    Query(pagination): Query<Pagination>,
) -> Result<Response, ServiceError<TopicServiceError>> {
    let topics = service.list(pagination).await?;

    let res = if topics.is_empty() {
        StatusCode::NO_CONTENT.into_response()
    } else {
        Json(topics).into_response()
    };
    Ok(res)
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
#[instrument(skip(service), err(Debug))]
pub async fn get_topic(
    State(service): State<TopicService>,
    Path(topic_id): Path<TopicId>,
) -> Result<Response, ServiceError<TopicServiceError>> {
    let topic = service.get(topic_id).await?;

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
#[instrument(skip_all, err(Debug), fields(req.name = topic.name, req.description = topic.description))]
async fn create_topic(
    State(service): State<TopicService>,
    Json(topic): Json<TopicRequest>,
) -> Result<TopicResponse, ServiceError<TopicServiceError>> {
    let new_topic = service.create(topic.name, topic.description).await?;
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
#[instrument(skip(service), err(Debug))]
pub async fn delete_topic(
    State(service): State<TopicService>,
    Path(topic_id): Path<TopicId>,
) -> Result<StatusCode, ServiceError<TopicServiceError>> {
    match service.delete(topic_id).await? {
        Some(_) => Ok(StatusCode::NO_CONTENT),
        None => Ok(StatusCode::NOT_FOUND),
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
#[instrument(skip(service, topic), err(Debug), fields(
    topic.name = topic.name,
    topic.desc = topic.description.as_ref().map_present_or(None, |d| Some(d.map(String::as_str).unwrap_or("null"))),
))]
pub async fn patch_topic(
    State(service): State<TopicService>,
    Path(topic_id): Path<TopicId>,
    Json(topic): Json<TopicPatchRequest>,
) -> Result<Response, ServiceError<TopicServiceError>> {
    let updated_topic = service
        .patch(topic_id, topic.name, topic.description)
        .await?;

    Ok(updated_topic
        .map(|t| Json(t).into_response())
        .unwrap_or_else(|| StatusCode::NOT_FOUND.into_response()))
}