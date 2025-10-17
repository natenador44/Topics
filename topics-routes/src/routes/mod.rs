use crate::error::TopicServiceError;
use crate::metrics;
use crate::routes::requests::{BulkCreateTopicRequest, TopicPatchRequest};
use crate::routes::responses::{BulkCreateResponse, TopicError};
use crate::service::{CreateManyTopic, PatchOutcome, TopicCreation, TopicService};
use crate::state::TopicAppState;
use axum::middleware::{self};
use axum::routing::patch;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response, Result},
    routing::{delete, get, post},
};
use engine::Pagination;
use engine::error::EndpointError;
use engine::list_criteria::ListFilter;
use engine::stream::StreamingResponse;
use requests::CreateTopicRequest;
use responses::TopicResponse;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use topics_core::TopicEngine;
use topics_core::list_filter::TopicFilter;
use topics_core::model::Topic;
use tracing::{info, instrument};
use utoipa::OpenApi;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

mod api_doc;
mod requests;
mod responses;

const TOPIC_ROOT_PATH: &str = "/topics";

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = TOPIC_ROOT_PATH, api = TopicDocs),
    )
)]
struct ApiDoc;

#[derive(OpenApi)]
#[openapi(paths(
    list_topics,
    get_topic,
    create_topic,
    bulk_create_topics,
    delete_topic,
    patch_topic,
))]
struct TopicDocs;

const DEFAULT_TOPIC_SEARCH_PAGE_SIZE: u64 = 25;

const TOPIC_LIST_PATH: &str = "/";
const TOPIC_GET_PATH: &str = "/{topic_id}";
const TOPIC_CREATE_PATH: &str = "/";
const TOPIC_BULK_CREATE_PATH: &str = "/bulk";
const TOPIC_DELETE_PATH: &str = "/{topic_id}";
const TOPIC_PATCH_PATH: &str = "/{topic_id}";

pub fn build<T: TopicEngine>(app_state: TopicAppState<T>) -> Router {
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(routes(app_state))
        .split_for_parts();

    router.merge(SwaggerUi::new("/topics/swagger-ui").url("/topics/api-docs/openapi.json", api))
}

fn routes<S, T: TopicEngine>(app_state: TopicAppState<T>) -> OpenApiRouter<S> {
    let main_router = OpenApiRouter::new()
        .route(TOPIC_LIST_PATH, get(list_topics))
        .route(TOPIC_GET_PATH, get(get_topic))
        .route(TOPIC_CREATE_PATH, post(create_topic))
        .route(TOPIC_BULK_CREATE_PATH, post(bulk_create_topics))
        .route(TOPIC_DELETE_PATH, delete(delete_topic))
        .route(TOPIC_PATCH_PATH, patch(patch_topic));

    let router = if app_state.metrics_enabled {
        info!("metrics enabled, setting up metrics handler");
        let metrics_recorder = metrics::setup_recorder();
        main_router
            .route("/metrics", get(|| async move { metrics_recorder.render() }))
            .route_layer(middleware::from_fn(metrics::track_http))
    } else {
        info!("metrics not enabled, setting up service unavailable metrics handler");
        main_router
            .route("/metrics", get(|| async { (StatusCode::SERVICE_UNAVAILABLE, "Metrics endpoint is disabled. Metrics must be enabled and the service restarted")}))
    };

    OpenApiRouter::new()
        .nest(TOPIC_ROOT_PATH, router)
        .with_state(app_state)
}

#[derive(Debug, ToSchema, Serialize, Deserialize, Copy, Clone, PartialEq, Eq)]
/// The type of the ID that identifies a Topic.
/// This changes depending on how the app is configured.
struct IdType;

type ResponseType = Topic<IdType>;

/// Get the topic associated with the given id.
// #[axum::debug_handler]
#[utoipa::path(
    get,
    path = TOPIC_LIST_PATH,
    responses(
        (status = OK, description = "Topics were found on the given page", body = Vec<ResponseType>),
        (status = NO_CONTENT, description = "No topics exist on the given page"),
    ),
    params(
        ("page" = u32, Query, description = "The offset page to start the listing with"),
        ("page_size" = u32, Query, description = "The max number of topics to return"),
    )
)]
#[instrument(skip(service), err(Debug), fields(req.page = pagination.page, req.page_size = pagination.page_size))]
pub async fn list_topics<T>(
    State(service): State<TopicService<T>>,
    Query(pagination): Query<Pagination>,
) -> Result<Response, EndpointError<TopicServiceError>>
where
    T: TopicEngine + Send + Sync + 'static,
{
    // TODO can list by name as well
    let topics = service
        .list(TopicFilter::criteria(
            pagination,
            DEFAULT_TOPIC_SEARCH_PAGE_SIZE,
        ))
        .await?;

    let res = if topics.is_empty() {
        StatusCode::NO_CONTENT.into_response()
    } else {
        StreamingResponse::ok(topics.into_iter().map(TopicResponse::ok)).into_response()
    };
    Ok(res)
}

/// Get the topic associated with the given id.
// #[axum::debug_handler]
#[utoipa::path(
    get,
    path = TOPIC_GET_PATH,
    responses(
        (status = OK, description = "A topic was found that matched the given TopicId", body = Topic<IdType>),
        (status = NOT_FOUND, description = "No topics with the given TopicId were found"),
    ),
    params(
        ("topic_id" = IdType, Path, description = "The TopicId to find"),
    )
)]
#[instrument(skip(service), err(Debug))]
pub async fn get_topic<T>(
    State(service): State<TopicService<T>>,
    Path(topic_id): Path<T::TopicId>,
) -> Result<Response, EndpointError<TopicServiceError>>
where
    T: TopicEngine,
{
    let topic = service.get(topic_id).await?;

    Ok(topic
        .map(|t| TopicResponse::ok(t).into_response())
        .unwrap_or_else(|| TopicError::not_found().into_response()))
}

/// Create a new Topic and return its ID
#[utoipa::path(
    post,
    path = TOPIC_CREATE_PATH,
    responses(
        (status = CREATED, description = "A topic was successfully created", body = TopicResponse<IdType>),
        (status = UNPROCESSABLE_ENTITY, description = "The name in the request was null"),
    ),
    request_body = CreateTopicRequest
)]
#[instrument(skip_all, err(Debug), fields(req.name = topic.name, req.description = topic.description))]
async fn create_topic<T>(
    State(service): State<TopicService<T>>,
    Json(topic): Json<CreateTopicRequest>,
) -> Result<Response, EndpointError<TopicServiceError>>
where
    T: TopicEngine,
{
    let new_topic = service
        .create(TopicCreation::new(topic.name, topic.description))
        .await?;
    Ok(TopicResponse::created(new_topic).into_response())
}

type BuildTopicCreateType = BulkCreateResponse<IdType>;

#[utoipa::path(
    post,
    path = TOPIC_BULK_CREATE_PATH,
    responses(
        (
            status = CREATED,
            description = "All topics were successfully created. The outcomes array will contain all 'Success' types", body = Vec<BuildTopicCreateType>,
            example = json!(api_doc::examples::create::bulk_all_success()),
        ),
        (
            status = MULTI_STATUS,
            description = "Some topics were successfully created, some were not. The outcomes array will contain a mix of 'Success' and 'Fail' types, an each 'Fail' outcome will have a failure reason",
            body = Vec<BuildTopicCreateType>,
            example = json!(api_doc::examples::create::bulk_mixed_success()),
        ),
        (
            status = UNPROCESSABLE_ENTITY,
            description = "None of the topics were able to be created. The outcomes array will contain only 'Fail' types with error reasons for each",
            body = Vec<BuildTopicCreateType>,
            example = json!(api_doc::examples::create::bulk_no_success()),
        ),
        (status = BAD_REQUEST, description = "An empty array was given", body = TopicError),
    ),
    request_body = Vec<CreateTopicRequest>
)]
#[instrument(skip_all, err(Debug), fields(req.topic_count = topics.len()))]
/// Create several topics at once, given the array of creation requests given in the request.
/// The outcomes array returned should contain the results of each request in the order they were received
async fn bulk_create_topics<T>(
    State(service): State<TopicService<T>>,
    Json(topics): Json<Vec<BulkCreateTopicRequest>>,
) -> Result<Response, EndpointError<TopicServiceError>>
where
    T: TopicEngine,
{
    if topics.is_empty() {
        return Ok(TopicError::bad_request("a non-empty array is required").into_response());
    }

    let topics = service
        .create_many(
            topics
                .into_iter()
                .map(|t| CreateManyTopic::new(t.name, t.description)),
        )
        .await?;

    Ok(BulkCreateResponse::new(topics).into_response())
}

// Delete the topic associated with the given id
#[utoipa::path(
    delete,
    path = TOPIC_DELETE_PATH,
    responses(
        (status = NO_CONTENT, description = "The topic was successfully deleted, or never existed"),
    ),
    params(
        ("topic_id" = IdType, Path, description = "The ID of the topic to delete to delete")
    )
)]
#[instrument(skip(service), err(Debug))]
pub async fn delete_topic<T>(
    State(service): State<TopicService<T>>,
    Path(topic_id): Path<T::TopicId>,
) -> Result<Response, EndpointError<TopicServiceError>>
where
    T: TopicEngine,
{
    match service.delete(topic_id).await? {
        Some(_) => Ok(StatusCode::NO_CONTENT.into_response()),
        None => Ok(TopicError::not_found().into_response()),
    }
}

/// Update the topic associated with the given id using the given information.
#[utoipa::path(
    patch,
    path = TOPIC_PATCH_PATH,
    responses(
        (status = OK, description = "The topic was successfully patched", body = Topic<IdType>),
        (status = UNPROCESSABLE_ENTITY, description = "'name' was set to null"),
        (status = NOT_FOUND, description = "The topic was not found so could not be updated"),
    ),
    params(
        ("topic_id" = IdType, Path, description = "The TopicId to patch")
    ),
    request_body = TopicPatchRequest,
)]
#[instrument(skip(service, topic), err(Debug), fields(
    topic.name = topic.name.as_ref().map_present_or(None, |d| Some(d.map(String::as_str).unwrap_or("null"))),
    topic.desc = topic.description.as_ref().map_present_or(None, |d| Some(d.map(String::as_str).unwrap_or("null"))),
))]
pub async fn patch_topic<T>(
    State(service): State<TopicService<T>>,
    Path(topic_id): Path<T::TopicId>,
    Json(topic): Json<TopicPatchRequest>,
) -> Result<Response, EndpointError<TopicServiceError>>
where
    T: TopicEngine,
{
    let outcome = service
        .patch(topic_id, topic.name, topic.description)
        .await?;

    let res = match outcome {
        PatchOutcome::Success(t) => TopicResponse::ok(t).into_response(),
        PatchOutcome::InvalidName => {
            TopicError::unprocessable_entity("name cannot be null").into_response()
        }
        PatchOutcome::NotFound => TopicError::not_found().into_response(),
    };

    Ok(res)
}
