use crate::error::{RepoResult, SetRepoError, TopicRepoError};
use crate::models::{Set, SetId, Topic, TopicId};
use crate::repository::sets::ExistingSetRepository;
use crate::repository::topics::ExistingTopicRepository;
use crate::search_filters::{SetSearchCriteria, TopicSearchCriteria};
use serde_json::Value;
use std::sync::Arc;

pub mod sets;
pub mod topics;

pub trait TopicsRepository {
    type ExistingTopic: ExistingTopicRepository + Send + Sync + 'static;
    /// Expect `topic_id` to exist so other operations can be done on it.
    /// This should check if `topic_id` exists, and if so, return an `ExistingTopicRepository`
    /// where you can delete, update, or do things with sets or identifiers for this topic
    fn expect_existing(
        &self,
        topic_id: TopicId,
    ) -> impl Future<Output = RepoResult<Option<Self::ExistingTopic>, TopicRepoError>> + Send;
    fn find(
        &self,
        topic_id: TopicId,
    ) -> impl Future<Output = RepoResult<Option<Topic>, TopicRepoError>> + Send;
    fn create(
        &self,
        name: String,
        description: Option<String>,
    ) -> impl Future<Output = RepoResult<Topic, TopicRepoError>> + Send;

    fn search(
        &self,
        topic_search_criteria: TopicSearchCriteria,
    ) -> impl Future<Output = RepoResult<Vec<Topic>, TopicRepoError>> + Send;
}

pub trait SetsRepository {
    type ExistingSet: ExistingSetRepository + Send + Sync + 'static;

    fn expect_existing(
        &self,
        set: SetId,
    ) -> impl Future<Output = RepoResult<Option<Self::ExistingSet>, SetRepoError>> + Send;

    fn find(
        &self,
        set_id: SetId,
    ) -> impl Future<Output = RepoResult<Option<Set>, SetRepoError>> + Send;

    fn create(
        &self,
        name: String,
        initial_entity_payloads: Vec<Value>,
    ) -> impl Future<Output = RepoResult<Set, SetRepoError>> + Send;

    fn search(
        &self,
        set_search_criteria: SetSearchCriteria,
    ) -> impl Future<Output = RepoResult<Vec<Set>, SetRepoError>> + Send;
}
pub trait EntitiesRepository {}
pub trait IdentifiersRepository {}

impl<T> TopicsRepository for Arc<T>
where
    T: TopicsRepository + Send + Sync,
{
    type ExistingTopic = T::ExistingTopic;

    async fn expect_existing(
        &self,
        topic_id: TopicId,
    ) -> RepoResult<Option<Self::ExistingTopic>, TopicRepoError> {
        (&**self).expect_existing(topic_id).await
    }

    async fn find(&self, topic_id: TopicId) -> RepoResult<Option<Topic>, TopicRepoError> {
        (&**self).find(topic_id).await
    }

    async fn create(
        &self,
        name: String,
        description: Option<String>,
    ) -> RepoResult<Topic, TopicRepoError> {
        (&**self).create(name, description).await
    }

    async fn search(
        &self,
        topic_search_criteria: TopicSearchCriteria,
    ) -> RepoResult<Vec<Topic>, TopicRepoError> {
        (&**self).search(topic_search_criteria).await
    }
}
