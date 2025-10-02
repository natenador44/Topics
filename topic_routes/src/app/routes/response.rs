use std::{marker::PhantomData, time::Duration};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_streams::StreamBodyAs;
use serde::Serialize;
use tokio_stream::StreamExt;

/// Can be used as the return type of an endpoint where
/// a transform needs to be done on an existing collection.
/// This can prevent unnecessary allocations into a new collection
/// of a different type before returning to the user.
/// Can also be used to "throttle" the output, i.e. put a delay between
/// when each element is streamed back to the user.
#[derive(Debug)]
pub struct StreamingResponse<T> {
    stream: StreamBodyAs<'static>,
    _phantom: PhantomData<T>,
}

impl<T> StreamingResponse<T>
where
    T: Serialize + Send + Sync + 'static,
{
    pub fn new<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: Send + Sync + 'static,
    {
        let stream = tokio_stream::iter(iter);
        Self {
            stream: StreamBodyAs::json_array(stream),
            _phantom: PhantomData,
        }
    }

    #[allow(unused)]
    pub fn with_throttle<I>(iter: I, throttle_mills: u64) -> Self
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: Send + Sync + 'static,
    {
        let stream = tokio_stream::iter(iter).throttle(Duration::from_millis(throttle_mills));
        Self {
            stream: StreamBodyAs::json_array(stream),
            _phantom: PhantomData,
        }
    }
}

impl<T> IntoResponse for StreamingResponse<T> {
    fn into_response(self) -> Response {
        (StatusCode::OK, self.stream).into_response()
    }
}
