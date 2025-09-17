use serde_json::Value;

use crate::{
    app::models::{Entity, TopicId, TopicSetId},
    error::{AppResult, SetServiceError},
};

#[derive(Debug, Clone)]
pub struct SetService<T> {
    repo: T,
}

impl<T> SetService<T>
where
    T: Clone,
{
    pub fn new(repo: T) -> Self {
        Self { repo }
    }

    pub fn create(
        &self,
        topic_id: TopicId,
        set_name: String,
        entities: Vec<Value>,
    ) -> AppResult<TopicSetId, SetServiceError> {
        todo!()
    }
}
