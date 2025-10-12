mod requests;
mod responses;
use crate::error::TopicServiceError;
use crate::service::TopicService;
use crate::state::TopicAppState;
use axum::extract::{MatchedPath, Request};
use axum::middleware::{self, Next};
use axum::routing::patch;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response, Result},
    routing::{delete, get, post},
};
use chrono::{DateTime, Utc};
use engine::error::ServiceError;
use engine::list_criteria::SearchFilter;
use engine::stream::StreamingResponse;
use engine::{Pagination, patch_field_schema};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use optional_field::{Field, serde_optional_fields};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::marker::PhantomData;
use tokio::time::Instant;
use topics_core::{Topic, TopicEngine, TopicFilter, TopicId};
use tracing::{debug, info, instrument};
use utoipa::OpenApi;
use utoipa::ToSchema;
use utoipa::schema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

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

const DEFAULT_TOPIC_SEARCH_PAGE_SIZE: u64 = 25;

const TOPIC_LIST_PATH: &str = "/";
const TOPIC_GET_PATH: &str = "/{topic_id}";
const TOPIC_CREATE_PATH: &str = "/";
const TOPIC_DELETE_PATH: &str = "/{topic_id}";
const TOPIC_PATCH_PATH: &str = "/{topic_id}";

pub fn build<T: TopicEngine>(app_state: TopicAppState<T>) -> Router {
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(routes(app_state))
        .split_for_parts();

    router.merge(SwaggerUi::new("/topics/swagger-ui").url("/topics/api-docs/openapi.json", api))
}

fn routes<S, T: TopicEngine>(app_state: TopicAppState<T>) -> OpenApiRouter<S> {
    let metrics_recorder = setup_metrics_recorder();

    OpenApiRouter::new()
        .nest(
            TOPIC_ROOT_PATH,
            OpenApiRouter::new()
                .route(TOPIC_LIST_PATH, get(list_topics))
                .route(TOPIC_GET_PATH, get(get_topic))
                .route(TOPIC_CREATE_PATH, post(create_topic))
                .route(TOPIC_DELETE_PATH, delete(delete_topic))
                .route(TOPIC_PATCH_PATH, patch(patch_topic))
                .route("/metrics", get(|| async move { metrics_recorder.render() }))
                .route_layer(middleware::from_fn(track_http_metrics)),
        )
        .with_state(app_state)
}

fn setup_metrics_recorder() -> PrometheusHandle {
    const EXPONENTIAL_SECONDS: &[f64] = &[0.005, 0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0];

    const REQ_RES_BUCKETS: &[f64] = &[128.0, 256.0, 512.0, 1024.0, 2048.0, 4096.0, 8192.0, 16384.0];

    PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full("http_requests_duration_seconds".to_string()),
            EXPONENTIAL_SECONDS,
        )
        .unwrap()
        .set_buckets_for_metric(
            Matcher::Full("http_request_size".to_string()),
            REQ_RES_BUCKETS,
        )
        .unwrap()
        .install_recorder()
        .unwrap()
}

async fn track_http_metrics(req: Request, next: Next) -> impl IntoResponse {
    // TODO figure out what "matched path" is
    let path = if let Some(matched_path) = req.extensions().get::<MatchedPath>() {
        matched_path.as_str().to_owned()
    } else {
        req.uri().path().to_owned()
    };

    if path.ends_with("metrics") {
        return next.run(req).await;
    }

    let method = req.method().clone();

    let req_size = req
        .headers()
        .get("Content-Length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok());

    if let Some(req_size) = req_size {
        metrics::histogram!("http_request_size").record(req_size as f64);
    }

    let start = Instant::now();
    let response = next.run(req).await;

    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    let req_size = response
        .headers()
        .get("Content-Length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok());

    if let Some(req_size) = req_size {
        metrics::histogram!("http_request_size").record(req_size as f64);
    }

    let labels = [
        ("method", method.to_string()),
        ("path", path),
        ("status", status),
    ];

    metrics::counter!("http_requests_total", &labels).increment(1);

    let histogram = metrics::histogram!("http_requests_duration_seconds", &labels);
    histogram.record(latency);
    response
}

#[derive(Debug, Serialize, ToSchema)]
struct TopicResponse<T> {
    #[serde(skip)]
    status_code: StatusCode,
    id: T,
    name: String,
    description: Option<String>,
    created: DateTime<Utc>,
    updated: Option<DateTime<Utc>>,
    #[serde(skip)]
    _phantom: PhantomData<T>,
}

impl<T> TopicResponse<T> {
    fn ok(topic: Topic<T>) -> Self {
        Self {
            status_code: StatusCode::OK,
            id: topic.id,
            name: topic.name,
            description: topic.description,
            created: topic.created,
            updated: topic.updated,
            _phantom: PhantomData,
        }
    }

    fn created(topic: Topic<T>) -> Self {
        Self {
            status_code: StatusCode::CREATED,
            id: topic.id,
            name: topic.name,
            description: topic.description,
            created: topic.created,
            updated: topic.updated,
            _phantom: PhantomData,
        }
    }
}

impl<T: TopicId> IntoResponse for TopicResponse<T> {
    fn into_response(self) -> Response {
        (self.status_code, Json(self)).into_response()
    }
}

#[derive(Debug, ToSchema, Serialize, Deserialize, Copy, Clone, PartialEq, Eq)]
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
#[instrument(skip(service), err(Debug))]
pub async fn list_topics<T>(
    State(service): State<TopicService<T>>,
    Query(pagination): Query<Pagination>,
) -> Result<Response, ServiceError<TopicServiceError>>
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
        metrics::counter!("topics_retrieved").increment(topics.len() as u64);
        StreamingResponse::new(topics.into_iter().map(TopicResponse::ok)).into_response()
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
) -> Result<Response, ServiceError<TopicServiceError>>
where
    T: TopicEngine,
{
    let topic = service.get(topic_id).await?;

    Ok(topic
        .map(|t| {
            metrics::counter!("topics_retrieved").increment(1);
            TopicResponse::ok(t).into_response()
        })
        .unwrap_or_else(|| StatusCode::NOT_FOUND.into_response()))
}

/// Create a new Topic and return its ID
#[utoipa::path(
    post,
    path = TOPIC_CREATE_PATH,
    responses(
        (status = CREATED, description = "A topic was successfully created", body = TopicResponse<IdType>),
    ),
    request_body = TopicRequest
)]
#[instrument(skip_all, err(Debug), fields(req.name = topic.name, req.description = topic.description))]
async fn create_topic<T>(
    State(service): State<TopicService<T>>,
    Json(topic): Json<TopicRequest>,
) -> Result<TopicResponse<T::TopicId>, ServiceError<TopicServiceError>>
where
    T: TopicEngine,
{
    let new_topic = service.create(topic.name, topic.description).await?;

    metrics::counter!("num_topics_created").increment(1);

    Ok(TopicResponse::created(new_topic))
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
) -> Result<StatusCode, ServiceError<TopicServiceError>>
where
    T: TopicEngine,
{
    match service.delete(topic_id).await? {
        Some(_) => {
            metrics::counter!("num_topics_deleted").increment(1);
            Ok(StatusCode::NO_CONTENT)
        }
        None => Ok(StatusCode::NOT_FOUND),
    }
}

/// Update the topic associated with the given id using the given information.
#[utoipa::path(
    patch,
    path = TOPIC_PATCH_PATH,
    responses(
        (status = OK, description = "The topic was successfully patched", body = Topic<IdType>),
        (status = NOT_FOUND, description = "The topic was not found so could not be updated"),
    ),
    params(
        ("topic_id" = IdType, Path, description = "The TopicId to patch")
    ),
    request_body = TopicPatchRequest,
)]
#[instrument(skip(service, topic), err(Debug), fields(
    topic.name = topic.name,
    topic.desc = topic.description.as_ref().map_present_or(None, |d| Some(d.map(String::as_str).unwrap_or("null"))),
))]
pub async fn patch_topic<T>(
    State(service): State<TopicService<T>>,
    Path(topic_id): Path<T::TopicId>,
    Json(topic): Json<TopicPatchRequest>,
) -> Result<Response, ServiceError<TopicServiceError>>
where
    T: TopicEngine,
{
    let updated_topic = service
        .patch(topic_id, topic.name, topic.description)
        .await?;

    Ok(updated_topic
        .map(|t| {
            metrics::counter!("num_topics_patched").increment(1);
            Json(t).into_response()
        })
        .unwrap_or_else(|| StatusCode::NOT_FOUND.into_response()))
}
