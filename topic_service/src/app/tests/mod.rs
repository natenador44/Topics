//! All tests in this module are intended to test the contract made by the API,
//! e.g. return codes, handling query parameters, handling path parameters.
use crate::app::models::{Entity, EntityId, Set, SetId, TopicId};
use crate::app::repository::SetRepoError;
use crate::app::services::{ResourceOutcome, SetSearchCriteria};
use crate::app::{
    models::Topic,
    repository::{
        IdentifierRepository, MockIdentifierRepository, MockSetRepository, MockTopicRepository,
        Repository, SetRepository, TopicFilter, TopicRepoError, TopicRepository,
    },
};
use crate::error::AppResult;
use serde_json::Value;
use std::{ops::Deref, sync::Arc};

mod v1;

#[derive(Debug, Clone)]
struct MockRepo {
    topic_repo: MockTopicRepoWrapper,
    set_repo: MockSetRepoWrapper,
}

impl MockRepo {
    fn for_topics_test(topic_repo: MockTopicRepository) -> Self {
        Self::new(topic_repo, MockSetRepository::new())
    }

    fn for_sets_test(set_repo: MockSetRepository, topic_repo: MockTopicRepository) -> Self {
        Self::new(topic_repo, set_repo)
    }

    fn new(topic_repo: MockTopicRepository, set_repo: MockSetRepository) -> Self {
        Self {
            topic_repo: MockTopicRepoWrapper(Arc::new(topic_repo)),
            set_repo: MockSetRepoWrapper(Arc::new(set_repo)),
        }
    }
}

impl Repository for MockRepo {
    type TopicRepo = MockTopicRepoWrapper;

    type IdentifierRepo = MockIdentifierRepoWrapper;

    type SetRepo = MockSetRepoWrapper;

    fn topics(&self) -> Self::TopicRepo {
        self.topic_repo.clone()
    }

    fn identifiers(&self) -> Self::IdentifierRepo {
        todo!()
    }

    fn sets(&self) -> Self::SetRepo {
        self.set_repo.clone()
    }
}

#[derive(Debug, Clone)]
struct MockTopicRepoWrapper(Arc<MockTopicRepository>);
impl Deref for MockTopicRepoWrapper {
    type Target = MockTopicRepository;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl TopicRepository for MockTopicRepoWrapper {
    async fn search(
        &self,
        page: usize,
        page_size: usize,
        filters: Vec<TopicFilter>,
    ) -> AppResult<Vec<Topic>, TopicRepoError> {
        self.0.search(page, page_size, filters).await
    }

    async fn get(&self, topic_id: TopicId) -> AppResult<Option<Topic>, TopicRepoError> {
        self.0.get(topic_id).await
    }

    async fn exists(&self, topic_id: TopicId) -> AppResult<bool, TopicRepoError> {
        self.0.exists(topic_id).await
    }

    async fn create(
        &self,
        name: String,
        description: Option<String>,
    ) -> AppResult<Topic, TopicRepoError> {
        self.0.create(name, description).await
    }

    async fn delete(&self, topic_id: TopicId) -> AppResult<ResourceOutcome, TopicRepoError> {
        self.0.delete(topic_id).await
    }

    async fn update(
        &self,
        topic_id: TopicId,
        name: Option<String>,
        description: Option<String>,
    ) -> AppResult<Option<Topic>, TopicRepoError> {
        self.0.update(topic_id, name, description).await
    }
}

#[derive(Debug, Clone)]
struct MockIdentifierRepoWrapper(Arc<MockIdentifierRepository>);
impl Deref for MockIdentifierRepoWrapper {
    type Target = MockIdentifierRepository;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl IdentifierRepository for MockIdentifierRepoWrapper {}

#[derive(Debug, Clone)]
struct MockSetRepoWrapper(Arc<MockSetRepository>);
impl Deref for MockSetRepoWrapper {
    type Target = MockSetRepository;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl SetRepository for MockSetRepoWrapper {
    async fn search(
        &self,
        topic_id: TopicId,
        search_criteria: SetSearchCriteria,
    ) -> AppResult<Vec<Set>, SetRepoError> {
        self.0.search(topic_id, search_criteria).await
    }

    async fn create(
        &self,
        topic_id: TopicId,
        set_name: String,
        initial_entity_payloads: Vec<Value>,
    ) -> AppResult<Set, SetRepoError> {
        self.0
            .create(topic_id, set_name, initial_entity_payloads)
            .await
    }

    async fn get(&self, topic_id: TopicId, set_id: SetId) -> AppResult<Set, SetRepoError> {
        self.0.get(topic_id, set_id).await
    }

    async fn delete(
        &self,
        topic_id: TopicId,
        set_id: SetId,
    ) -> AppResult<ResourceOutcome, SetRepoError> {
        self.0.delete(topic_id, set_id).await
    }
}
