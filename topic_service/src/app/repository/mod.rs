use std::fmt::Debug;

use error_stack::{Result, report};

use crate::{app::models::Topic, error::InitError};

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
}

#[derive(Debug)]
pub enum TopicFilter {
    Name(String),
    Description(String),
}

pub trait TopicRepository {
    fn search(
        &self,
        page: usize,
        page_size: usize,
        filters: Vec<TopicFilter>, // TODO find a way to not allocate memory with each request
    ) -> impl std::future::Future<Output = Result<Vec<Topic>, TopicRepoError>> + Send;
}

pub trait IdentifierRepository {}

pub trait SetRepository {}
