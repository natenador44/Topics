use ids::Id;
use list_filter::TopicListCriteria;
use model::{NewTopic, PatchTopic, Topic};
use result::{OptRepoResult, RepoResult};
use serde::Serialize;
use std::fmt::Debug;
use utoipa::ToSchema;

pub mod list_filter;
pub mod model;
pub mod result;

pub trait TopicEngine: Clone + Send + Sync + 'static {
    type TopicId: Id;
    type Repo: TopicRepository<TopicId = Self::TopicId>;
    // type Cache // bound not necessarily from this crate, since this will be common to all services

    fn repo(&self) -> Self::Repo;
}

// more reasons can be added, for example if we end up having restrictions on name or description
#[derive(Debug, Serialize, ToSchema, Copy, Clone, PartialEq, Eq)]
pub enum CreateManyFailReason {
    ServiceError,
    MissingName,
}

#[derive(Debug, Serialize, ToSchema, Clone, PartialEq, Eq)]
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

pub trait TopicRepository: Send + Sync + Clone + 'static {
    type TopicId: Id;

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
        topics: Vec<NewTopic>,
    ) -> impl Future<Output = RepoResult<Vec<RepoResult<Topic<Self::TopicId>>>>> + Send;

    fn patch(
        &self,
        id: Self::TopicId,
        patch: PatchTopic,
    ) -> impl Future<Output = OptRepoResult<Topic<Self::TopicId>>> + Send;

    fn delete(&self, id: Self::TopicId) -> impl Future<Output = OptRepoResult<()>> + Send;
}
