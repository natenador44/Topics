use axum::{
    Json,
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use topic_core::v1::TopicId;
use tracing::debug;
use uuid::Uuid;

use crate::app::pagination::Pagination;

#[derive(Debug, Deserialize)]
pub struct TopicRequest {
    name: String,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TopicSearch {
    name: Option<String>,
    description: Option<String>,
}

/// Query and filter for `Topic`s. Can specify pagination (page and page_size) to limit results returned.
/// Can also specify `SearchCriteria` to further reduce results based on name, description, or more.
pub async fn search(
    Query(pagination): Query<Pagination>,
    Query(search): Query<TopicSearch>,
) -> impl IntoResponse {
    debug!("topic search -> pagination: {pagination:?}, search: {search:?}")
}

/// Get the `Topic` associated with the given `TopicId`.
/// Returns 404 if the topic does not exist, otherwise 200 with a `Json<Topic>` payload.
pub async fn get(Path(topic_id): Path<TopicId>) -> impl IntoResponse {
    debug!("topic get -> topic_id: {topic_id}");
}

/// Create a new `Topic` and return the new `Topic`'s `TopicId`, with a 201 CREATED status code
pub async fn create(Json(topic): Json<TopicRequest>) -> impl IntoResponse {
    debug!("topic create -> new topic: {topic:?}");

    Json(Uuid::new_v4()) // not sure if this should be JSON but may as well be consistent right now
}

/// Delete the `Topic` associated with the given `TopicId`, returning a 204 NO_CONTENT status code
pub async fn delete(Path(topic_id): Path<TopicId>) -> impl IntoResponse {
    debug!("topic delete -> topic id: {topic_id}");
    StatusCode::NO_CONTENT
}

/// Update the `Topic` associated with the given `TopicId` using the given `Topic` information.
/// Returns the updated version of the `Topic` if the `topic_id` exists, otherwise a 404 NOT FOUND
pub async fn update(
    Path(topic_id): Path<TopicId>,
    Json(topic): Json<TopicRequest>,
) -> impl IntoResponse {
    debug!("topic upsert -> topic id: {topic_id}, updates: {topic:?}");
}

// TODO do we want a PATCH?
