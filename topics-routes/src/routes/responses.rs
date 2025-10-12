use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::marker::PhantomData;
use topics_core::TopicId;
use topics_core::model::Topic;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct TopicResponse<T> {
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
    pub fn ok(topic: Topic<T>) -> Self {
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

    pub fn created(topic: Topic<T>) -> Self {
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
