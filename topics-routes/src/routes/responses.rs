use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use std::borrow::Cow;
use topics_core::model::Topic;
use topics_core::{CreateManyTopicStatus, TopicId};
use tracing::warn;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct TopicResponse<T> {
    #[serde(skip)]
    status_code: StatusCode,
    #[serde(flatten)]
    topic: Topic<T>,
}

impl<T> TopicResponse<T> {
    pub fn ok(topic: Topic<T>) -> Self {
        Self {
            status_code: StatusCode::OK,
            topic,
        }
    }

    pub fn created(topic: Topic<T>) -> Self {
        Self {
            status_code: StatusCode::CREATED,
            topic,
        }
    }
}

impl<T: TopicId> IntoResponse for TopicResponse<T> {
    fn into_response(self) -> Response {
        (self.status_code, Json(self)).into_response()
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BulkCreateResponse<T> {
    #[serde(skip)]
    status_code: StatusCode,
    created: usize,
    failed: usize,
    outcomes: Vec<CreateManyTopicStatus<T>>, // can these be streamed? maybe if I rearrange things and don't have all this 'header' info
}

impl<T> BulkCreateResponse<T> {
    pub fn new(outcomes: Vec<CreateManyTopicStatus<T>>) -> Self {
        let (created, failed) =
            outcomes
                .iter()
                .fold((0, 0), |(created, failed), outcome| match outcome {
                    CreateManyTopicStatus::Success(_) => (created + 1, failed),
                    CreateManyTopicStatus::Fail { .. } => (created, failed + 1),
                    &CreateManyTopicStatus::Pending { .. } => {
                        warn!("one topic left in Pending status, counting as failed");
                        (created, failed + 1)
                    }
                });

        let status_code = match (created, failed) {
            (0, 1..) => StatusCode::UNPROCESSABLE_ENTITY,
            (1.., 0) => StatusCode::CREATED,
            _ => StatusCode::MULTI_STATUS,
        };

        Self {
            status_code,
            created,
            failed,
            outcomes,
        }
    }
}

impl<T: TopicId> IntoResponse for BulkCreateResponse<T> {
    fn into_response(self) -> Response {
        (self.status_code, Json(self)).into_response()
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TopicError<T = ()> {
    #[serde(skip)]
    status_code: StatusCode,
    message: Cow<'static, str>,
    #[serde(flatten)]
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
}

pub type ErrorMessageType = Cow<'static, str>;

impl TopicError<()> {
    pub fn not_found() -> Self {
        Self::new(
            StatusCode::NOT_FOUND,
            "the requested topic does not exist",
            None,
        )
    }

    pub fn bad_request(message: impl Into<ErrorMessageType>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message.into(), None)
    }

    pub fn unprocessable_entity(message: impl Into<ErrorMessageType>) -> Self {
        Self::new(StatusCode::UNPROCESSABLE_ENTITY, message.into(), None)
    }
}

impl<T: Serialize> TopicError<T> {
    // pub fn with_data(
    //     status_code: StatusCode,
    //     message: impl Into<ErrorMessageType>,
    //     data: T,
    // ) -> Self {
    //     Self {
    //         status_code,
    //         message: message.into(),
    //         data: Some(data),
    //     }
    // }

    pub fn new(
        status_code: StatusCode,
        message: impl Into<ErrorMessageType>,
        data: Option<T>,
    ) -> Self {
        Self {
            status_code,
            message: message.into(),
            data,
        }
    }
}

impl<T: Serialize> IntoResponse for TopicError<T> {
    fn into_response(self) -> Response {
        (self.status_code, Json(self)).into_response()
    }
}
