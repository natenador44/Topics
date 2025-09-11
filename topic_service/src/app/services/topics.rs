use error_stack::{Result, report};
use std::sync::Arc;

use repository::{Repository, TopicRepository};

use crate::{
    app::{
        models::Topic,
        pagination::{self, Pagination},
    },
    error::TopicServiceError,
};

#[derive(Debug)]
pub struct TopicService<T> {
    topic_repo: T,
}

impl<T> Clone for TopicService<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            topic_repo: self.topic_repo.clone(),
        }
    }
}

impl<T: Repository> TopicService<T> {
    pub fn new(repo: T) -> TopicService<T> {
        TopicService { topic_repo: repo }
    }

    pub fn search(
        &self,
        name: Option<&str>,
        description: Option<&str>,
        pagination: Pagination,
    ) -> Result<Vec<Topic>, TopicServiceError> {
        Err(report!(TopicServiceError::Test))
    }
}
