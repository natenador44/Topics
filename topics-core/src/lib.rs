use list_filter::TopicListCriteria;
use model::{NewTopic, PatchTopic, Topic};
use result::{OptRepoResult, RepoResult};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use utoipa::ToSchema;

pub mod list_filter;
pub mod model;
pub mod result;

pub trait TopicEngine: Clone + Send + Sync + 'static {
    type TopicId: TopicId;
    type Repo: TopicRepository<TopicId = Self::TopicId> + Send + Sync + 'static;
    // type Cache // bound not necessarily from this crate, since this will be common to all services

    fn repo(&self) -> Self::Repo;
}

// more reasons can be added, for example if we end up having restrictions on name or description
#[derive(Debug, Serialize, ToSchema)]
pub enum CreateManyFailReason {
    ServiceError,
    MissingName,
}

#[derive(Debug, Serialize, ToSchema)]
pub enum CreateManyTopicStatus<T> {
    Pending {
        name: String,
        description: Option<String>,
    },
    Success(Topic<T>),
    Fail {
        topic_name: Option<String>,
        topic_description: Option<String>,
        reason: CreateManyFailReason,
    },
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

    fn create_many(
        &self,
        topics: Vec<CreateManyTopicStatus<Self::TopicId>>,
    ) -> impl Future<Output = RepoResult<Vec<CreateManyTopicStatus<Self::TopicId>>>> + Send;

    fn patch(
        &self,
        id: Self::TopicId,
        patch: PatchTopic,
    ) -> impl Future<Output = OptRepoResult<Topic<Self::TopicId>>> + Send;

    fn delete(&self, id: Self::TopicId) -> impl Future<Output = OptRepoResult<()>> + Send;
}

pub trait TopicId:
    Debug + Send + Sync + Serialize + for<'de> Deserialize<'de> + Clone + ToSchema + PartialEq
{
}

impl<T> TopicId for T where
    T: Debug + Send + Sync + Serialize + for<'de> Deserialize<'de> + Clone + ToSchema + PartialEq
{
}
