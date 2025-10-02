use engine::error::{RepoResult, SetRepoError, TopicRepoError};
use engine::models::{Set, SetId, Topic, TopicId};
use engine::repository::topics::{ExistingTopicRepository, TopicUpdate};
use engine::repository::{EntitiesRepository, IdentifiersRepository, SetsRepository, TopicsRepository};
use engine::repository::sets::{ExistingSetRepository, SetUpdate};
use engine::search_filters::{SetSearchCriteria, TopicSearchCriteria};

pub struct TopicRepo; // TODO postgres pool

impl TopicsRepository for TopicRepo {
    type ExistingTopic = ExistingTopicRepo;

    async fn expect_existing(&self, topic_id: TopicId) -> RepoResult<Option<Self::ExistingTopic>, TopicRepoError> {
        todo!()
    }

    async fn find(&self, topic_id: TopicId) -> RepoResult<Option<Topic>, TopicRepoError> {
        todo!()
    }

    async fn create(&self, name: String, description: Option<String>) -> RepoResult<Topic, TopicRepoError> {
        todo!()
    }

    async fn search(&self, topic_search_criteria: TopicSearchCriteria) ->RepoResult<Vec<Topic>, TopicRepoError> {
        todo!()
    }
}

pub struct ExistingTopicRepo; // TODO postgres pool

impl ExistingTopicRepository for ExistingTopicRepo {
    type SetRepo = SetRepo;
    type IdentifierRepo = IdentifierRepo;

    fn sets(&self) -> Self::SetRepo {
        todo!()
    }

    fn identifiers(&self) -> Self::IdentifierRepo {
        todo!()
    }

    async fn delete(&self) -> RepoResult<(), TopicRepoError> {
        todo!()
    }

    async fn update(&self, topic: TopicUpdate) -> RepoResult<Topic, TopicRepoError> {
        todo!()
    }
}

pub struct SetRepo; // TODO postgres pool

impl SetsRepository for SetRepo {
    type ExistingSet = ExistingSetRepo;

    async fn expect_existing(&self, set: SetId) -> RepoResult<Option<Self::ExistingSet>, SetRepoError> {
        todo!()
    }

    async fn find(&self, set_id: SetId) -> RepoResult<Option<Set>, SetRepoError> {
        todo!()
    }

    async fn create(&self, name: String, initial_entity_payloads: Vec<serde_json::value::Value>) -> RepoResult<Set, SetRepoError> {
        todo!()
    }

    async fn search(&self, set_search_criteria: SetSearchCriteria) -> RepoResult<Vec<Set>, SetRepoError> {
        todo!()
    }
}

pub struct ExistingSetRepo; // TODO postgres pool

impl ExistingSetRepository for ExistingSetRepo {
    type EntitiesRepo = EntityRepo;

    fn entities(&self) -> Self::EntitiesRepo {
        todo!()
    }

    async fn delete(&self) -> RepoResult<(), SetRepoError> {
        todo!()
    }

    async fn update(&self, set: SetUpdate) -> RepoResult<Set, SetRepoError> {
        todo!()
    }
}

pub struct EntityRepo; // TODO postgres pool

impl EntitiesRepository for EntityRepo {}

pub struct IdentifierRepo; // TODO postgres pool

impl IdentifiersRepository for IdentifierRepo {}