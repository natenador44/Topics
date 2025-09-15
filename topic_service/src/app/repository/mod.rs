use std::fmt::Debug;

use error_stack::Result;

use crate::app::models::{Topic, TopicId};

pub mod file;

pub trait Repository: Clone + Send + Sync + Debug {
    type TopicRepo: TopicRepository + Send + Sync + 'static;
    type IdentifierRepo: IdentifierRepository + Send + Sync + 'static;
    type SetRepo: SetRepository + Send + Sync + 'static;

    fn topics(&self) -> Self::TopicRepo;
    fn identifiers(&self) -> Self::IdentifierRepo;
    fn sets(&self) -> Self::SetRepo;
}

#[derive(Debug, thiserror::Error)]
pub enum TopicRepoError {
    #[error("failed to search topics")]
    Search,
    #[error("error occurred while finding topic")]
    Get,
    #[error("failed to create new topic")]
    Create,
    #[error("failed to delete topic")]
    Delete,
    #[error("failed to update topic")]
    Update,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TopicFilter {
    Name(String),
    Description(String),
}

#[cfg_attr(test, mockall::automock)]
pub trait TopicRepository {
    fn search(
        &self,
        page: usize,
        page_size: usize,
        filters: Vec<TopicFilter>, // TODO find a way to not allocate memory with each request
    ) -> impl Future<Output = Result<Vec<Topic>, TopicRepoError>> + Send;

    fn get(
        &self,
        topic_id: TopicId,
    ) -> impl Future<Output = Result<Option<Topic>, TopicRepoError>> + Send;

    fn create(
        &self,
        name: String,
        description: Option<String>,
    ) -> impl Future<Output = Result<TopicId, TopicRepoError>> + Send;

    fn delete(&self, topic_id: TopicId) -> impl Future<Output = Result<(), TopicRepoError>> + Send;

    fn update(
        &self,
        topic_id: TopicId,
        name: Option<String>,
        description: Option<String>,
    ) -> impl Future<Output = Result<Option<Topic>, TopicRepoError>> + Send;
}

#[cfg_attr(test, mockall::automock)]
pub trait IdentifierRepository {}

#[cfg_attr(test, mockall::automock)]
pub trait SetRepository {}
