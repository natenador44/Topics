//! All tests in this module are intended to test the contract made by the API,
//! e.g. return codes, handling query parameters, handling path parameters.
use engine::Engine;
use engine::error::{RepoResult, SetRepoError, TopicRepoError};
use engine::models::{Set, SetId, Topic, TopicId};
use engine::repository::sets::{ExistingSetRepository, SetUpdate};
use engine::repository::topics::{ExistingTopicRepository, TopicUpdate};
use engine::repository::{
    EntitiesRepository, IdentifiersRepository, SetsRepository, TopicsRepository,
};
use engine::search_filters::{SetSearchCriteria, TopicSearchCriteria};
use mockall::automock;
use serde_json::Value;
use std::{ops::Deref, sync::Arc};

mod v1;

#[derive(Debug, Clone)]
struct TestEngine {
    repo: Arc<MockTopicRepo>,
}

impl TestEngine {
    fn new(repo: MockTopicRepo) -> Self {
        Self {
            repo: Arc::new(repo),
        }
    }
}

impl Engine for TestEngine {
    type Repo = Arc<MockTopicRepo>;

    fn topics(&self) -> Self::Repo {
        Arc::clone(&self.repo)
    }
}

#[derive(Debug, Clone)]
struct TopicRepo;

#[automock]
impl TopicsRepository for TopicRepo {
    type ExistingTopic = MockExistingTopicRepo;

    async fn expect_existing(
        &self,
        topic_id: TopicId,
    ) -> RepoResult<Option<Self::ExistingTopic>, TopicRepoError> {
        unreachable!()
    }

    async fn find(&self, topic_id: TopicId) -> RepoResult<Option<Topic>, TopicRepoError> {
        unreachable!()
    }

    async fn create(
        &self,
        name: String,
        description: Option<String>,
    ) -> RepoResult<Topic, TopicRepoError> {
        unreachable!()
    }

    async fn search(
        &self,
        topic_search_criteria: TopicSearchCriteria,
    ) -> RepoResult<Vec<Topic>, TopicRepoError> {
        unreachable!()
    }
}

struct ExistingTopicRepo;

#[automock]
impl ExistingTopicRepository for ExistingTopicRepo {
    type SetRepo = MockSetRepo;
    type IdentifierRepo = MockIdentifierRepo;

    fn sets(&self) -> Self::SetRepo {
        unreachable!()
    }

    fn identifiers(&self) -> Self::IdentifierRepo {
        unreachable!()
    }

    async fn delete(&self) -> RepoResult<(), TopicRepoError> {
        unreachable!()
    }

    async fn update(&self, topic: TopicUpdate) -> RepoResult<Topic, TopicRepoError> {
        unreachable!()
    }
}

struct SetRepo;

#[automock]
impl SetsRepository for SetRepo {
    type ExistingSet = MockExistingSetRepo;

    async fn expect_existing(
        &self,
        set: SetId,
    ) -> RepoResult<Option<Self::ExistingSet>, SetRepoError> {
        unreachable!()
    }

    async fn find(&self, set_id: SetId) -> RepoResult<Option<Set>, SetRepoError> {
        unreachable!()
    }

    async fn create(
        &self,
        name: String,
        description: Option<String>,
        initial_entity_payloads: Vec<Value>,
    ) -> RepoResult<Set, SetRepoError> {
        unreachable!()
    }

    async fn search(
        &self,
        set_search_criteria: SetSearchCriteria,
    ) -> RepoResult<Vec<Set>, SetRepoError> {
        unreachable!()
    }
}

struct ExistingSetRepo;

#[automock]
impl ExistingSetRepository for ExistingSetRepo {
    type EntitiesRepo = MockEntityRepo;

    fn entities(&self) -> Self::EntitiesRepo {
        unreachable!()
    }

    async fn delete(&self) -> RepoResult<(), SetRepoError> {
        unreachable!()
    }

    async fn update(&self, set: SetUpdate) -> RepoResult<Set, SetRepoError> {
        unreachable!()
    }
}

struct EntityRepo;

#[automock]
impl EntitiesRepository for EntityRepo {}

struct IdentifierRepo;

#[automock]
impl IdentifiersRepository for IdentifierRepo {}
