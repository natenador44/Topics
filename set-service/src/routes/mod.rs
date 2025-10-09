use engine::patch_field_schema;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Json, Router};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, patch, post};
use chrono::{DateTime, Utc};
use optional_field::Field;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use utoipa::{OpenApi, ToSchema};
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;
use engine::error::ServiceError;
use engine::id::{SetId, TopicId};
use engine::Pagination;
use crate::error::SetServiceError;
use crate::model::Set;
use crate::service::SetService;
use crate::state::SetAppState;

mod responses;
mod requests;
const DEFAULT_SET_PAGE_SIZE: u32 = 10;
const DEFAULT_ENTITY_PAGE_SIZE: u32 = 10;

const MISSING_RESOURCE_RESPONSE_BODY: &str =
    "The requested set resource does not exist";

const SET_ROOT_PATH: &str = "/sets";

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = SET_ROOT_PATH, api = SetDocs),
    )
)]
struct ApiDoc;

#[derive(OpenApi)]
#[openapi(paths(
    create_set,
    get_set,
    list_sets,
    delete_set,
    patch_set,
))]
pub struct SetDocs;

#[derive(Debug, Deserialize, ToSchema)]
pub struct SetRequest {
    name: String,
    description: Option<String>,
}

const CREATE_SET_PATH: &str = "/";
const GET_SET_PATH: &str = "/{set_id}";
const SET_LIST_PATH: &str = "/";
const DELETE_SET_PATH: &str = "/{set_id}";
const PATCH_SET_PATH: &str = "/{set_id}";

pub fn build(app_state: SetAppState) -> Router {
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(routes(app_state))
        .split_for_parts();

    router.merge(SwaggerUi::new("/sets/swagger-ui").url("/sets/api-docs/openapi.json", api))
}

pub fn routes<S>(app_state: SetAppState) -> OpenApiRouter<S> {
    OpenApiRouter::new()
        .nest(
            SET_ROOT_PATH,
            OpenApiRouter::new()
                .route(CREATE_SET_PATH, post(create_set))
                .route(GET_SET_PATH, get(get_set))
                .route(SET_LIST_PATH, get(list_sets))
                .route(DELETE_SET_PATH, delete(delete_set))
                .route(PATCH_SET_PATH, patch(patch_set))
        )
        .with_state(app_state)
}

#[derive(Debug, ToSchema, Serialize)]
struct SetResponse {
    #[serde(skip)]
    status_code: StatusCode,
    id: SetId,
    topic_id: TopicId,
    name: String,
    description: Option<String>,
    created: DateTime<Utc>,
    updated: Option<DateTime<Utc>>,
}

impl SetResponse {
    fn ok(set: Set) -> Self {
        Self::new(set, StatusCode::OK)
    }

    fn created(set: Set) -> Self {
        Self::new(set, StatusCode::CREATED)
    }

    fn new(set: Set, status_code: StatusCode) -> Self {
        Self {
            status_code,
            id: set.id,
            topic_id: set.topic_id,
            name: set.name,
            description: set.description,
            created: set.created,
            updated: set.updated,
        }
    }
}

impl IntoResponse for SetResponse {
    fn into_response(self) -> Response {
        (self.status_code, Json(self)).into_response()
    }
}

#[utoipa::path(
    post,
    path = CREATE_SET_PATH,
    responses(
        (status = CREATED, description = "A set was successfully created", body = SetResponse),
        (status = NOT_FOUND, description = "The set id does not exist"),
    ),
    params(
        ("set_id" = TopicId, Path, description = "The set ID associated with the new set")
    ),
    request_body = SetRequest,
)]
#[instrument(skip(service, set_request), ret, err(Debug), fields(
    req.name = set_request.name,
    req.desc = set_request.description,
))]
async fn create_set(
    State(service): State<SetService>,
    Path(set_id): Path<TopicId>,
    Json(set_request): Json<SetRequest>,
) -> Result<SetResponse, ServiceError<SetServiceError>> {
    let new_set = service
        .create(
            set_id,
            set_request.name,
            set_request.description,
        )
        .await?;

    Ok(SetResponse::created(new_set))
}

#[utoipa::path(
    get,
    path = GET_SET_PATH,
    responses(
        (status = OK, description = "Set was found", body = Vec<Set>),
        (status = NOT_FOUND, description = "The set id or the set id does not exist")
    ),
    params(
        ("set_id" = SetId, Path, description = "The set to get")
    ),
)]
// #[axum::debug_handler]
#[instrument(skip(service), ret, err(Debug))]
async fn get_set(
    State(service): State<SetService>,
    Path(set_id): Path<SetId>,
) -> Result<Response, ServiceError<SetServiceError>> {
    let set = service.get(set_id).await?;

    let res = set.map(|s| SetResponse::ok(s).into_response()).unwrap_or_else(|| (StatusCode::NOT_FOUND, MISSING_RESOURCE_RESPONSE_BODY).into_response());

    Ok(res)
}

/// Search through all Sets under a given Topic.
/// Sets can be searched through by name, or by certain identifiers assigned to their entities.
/// This differs from the entity search because this searches through all Sets instead of just one.
/// This also returns a list of Sets that match the search criteria, where the entity search returns
/// a list of Entities.
#[utoipa::path(
    get,
    path = SET_LIST_PATH,
    responses(
        (status = OK, description = "Sets were found", body = Vec<Set>),
        (status = NO_CONTENT, description = "No sets were found"),
        (status = NOT_FOUND, description = "The set id does not exist")
    ),
    params(
        ("set_id" = TopicId, Path, description = "Sets under this set will be searched"),
    ),
)]
// #[axum::debug_handler]
#[instrument(skip(service, pagination), ret, err(Debug), fields(
    req.page = pagination.page,
    req.page_size = pagination.page_size,
))]
async fn list_sets(
    State(service): State<SetService>,
    Query(pagination): Query<Pagination>,
) -> Result<Response, ServiceError<SetServiceError>> {
    let sets = service.list(pagination).await?;

    let res = if sets.is_empty() {
        StatusCode::NO_CONTENT.into_response()
    } else {
        Json(sets).into_response()
    };
    Ok(res)
}


#[utoipa::path(
    delete,
    path = DELETE_SET_PATH,
    responses(
        (status = NO_CONTENT, description = "The set was deleted"),
        (status = NOT_FOUND, description = "The set id or set id does not exist"),
    ),
    params(
        ("set_id" = SetId, Path, description = "The set to delete")
    ),
)]
#[instrument(skip(service), ret, err(Debug))]
async fn delete_set(
    State(service): State<SetService>,
    Path(set_id): Path<SetId>,
) -> Result<Response, ServiceError<SetServiceError>> {
    match service.delete(set_id).await? {
        Some(_) => Ok(StatusCode::NO_CONTENT.into_response()),
        None => {
            Ok((StatusCode::NOT_FOUND, MISSING_RESOURCE_RESPONSE_BODY).into_response())
        }
    }
}

#[derive(Debug, ToSchema, Deserialize)]
struct SetPatchRequest {
    name: Option<String>,
    #[schema(schema_with = patch_field_schema)]
    description: Field<String>,
}

/// Update the set associated with the given id using the given information.
#[utoipa::path(
    patch,
    path = PATCH_SET_PATH,
    responses(
        (status = OK, description = "The set was successfully patched", body = Set),
        (status = NOT_FOUND, description = "The set was not found so could not be updated"),
    ),
    params(
        ("set_id" = SetId, Path, description = "The SetId to patch")
    ),
    request_body = SetPatchRequest,
)]
#[instrument(skip(service), err(Debug), fields(
    set.name = set_patch.name,
    set.desc = set_patch.description.as_ref().map_present_or(None, |d| Some(d.map(String::as_str).unwrap_or("null"))),
))]
async fn patch_set(
    State(service): State<SetService>,
    Path(set_id): Path<SetId>,
    Json(set_patch): Json<SetPatchRequest>,
) -> axum::response::Result<Response, ServiceError<SetServiceError>> {
    let updated_set = service
        .patch(set_id, set_patch.name, set_patch.description)
        .await?;

    Ok(updated_set
        .map(|t| Json(t).into_response())
        .unwrap_or_else(|| (StatusCode::NOT_FOUND, MISSING_RESOURCE_RESPONSE_BODY).into_response()))
}
