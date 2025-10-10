use chrono::{DateTime, Utc};
use engine::Pagination;
use engine::list_criteria::{ListCriteria, SearchFilter, Tag};
use error_stack::Report;
use optional_field::Field;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use utoipa::ToSchema;

pub type RepoResult<T> = Result<T, Report<TopicRepoError>>;
pub type OptRepoResult<T> = Result<Option<T>, Report<TopicRepoError>>;
#[derive(Debug, thiserror::Error)]
pub enum TopicRepoError {
    #[error("failed to get topic")]
    Get,
    #[error("failed to list topics")]
    List,
    #[error("failed to create topic")]
    Create,
    #[error("failed to patch topic")]
    Patch,
    #[error("failed to delete topic")]
    Delete,
}

const MAX_FILTER_COUNT: usize = 1;
pub type TopicListCriteria = ListCriteria<TopicFilter, MAX_FILTER_COUNT>;
pub enum TopicFilter {
    Name(String),
}

impl SearchFilter for TopicFilter {
    const MAX_FILTER_COUNT: usize = MAX_FILTER_COUNT;
    type Criteria = TopicListCriteria;

    fn tag(&self) -> Tag {
        match self {
            TopicFilter::Name(_) => Tag::One,
        }
    }

    fn criteria(pagination: Pagination, default_page_size: u64) -> Self::Criteria {
        TopicListCriteria::new(pagination, default_page_size)
    }
}

pub trait TopicId:
    Debug + Send + Sync + Serialize + for<'de> Deserialize<'de> + Clone + ToSchema + PartialEq
{
}

impl<T> TopicId for T where
    T: Debug + Send + Sync + Serialize + for<'de> Deserialize<'de> + Clone + ToSchema + PartialEq
{
}

pub struct NewTopic {
    pub name: String,
    pub description: Option<String>,
}

impl NewTopic {
    pub fn new(name: String, description: Option<String>) -> Self {
        Self { name, description }
    }
}

pub struct PatchTopic {
    pub name: Option<String>,
    pub description: Field<String>,
}

impl PatchTopic {
    pub fn new(name: Option<String>, description: Field<String>) -> Self {
        Self { name, description }
    }
}

pub trait TopicEngine: Clone + Send + Sync + 'static {
    type TopicId: TopicId;
    type Repo: TopicRepository<TopicId = Self::TopicId> + Send + Sync + 'static;
    // type Cache // bound not necessarily from this crate, since this will be common to all services

    fn repo(&self) -> Self::Repo;
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Topic<T> {
    pub id: T,
    pub name: String,
    pub description: Option<String>,
    pub created: DateTime<Utc>,
    pub updated: Option<DateTime<Utc>>,
}

pub trait TopicRepository {
    type TopicId: TopicId;

    fn get(
        &self,
        id: Self::TopicId,
    ) -> impl Future<Output = OptRepoResult<Topic<Self::TopicId>>> + Send;

    fn list(
        &self,
        list_criteria: TopicListCriteria,
    ) -> impl Future<Output = RepoResult<Vec<Topic<Self::TopicId>>>> + Send;

    fn create(
        &self,
        new_topic: NewTopic,
    ) -> impl Future<Output = RepoResult<Topic<Self::TopicId>>> + Send;

    fn patch(
        &self,
        id: Self::TopicId,
        patch: PatchTopic,
    ) -> impl Future<Output = OptRepoResult<Topic<Self::TopicId>>> + Send;

    fn delete(&self, id: Self::TopicId) -> impl Future<Output = OptRepoResult<()>> + Send;
}
