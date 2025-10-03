use std::sync::Arc;

use engine::error::{RepoResult, SetRepoError, TopicRepoError};
use engine::models::{Set, SetId, Topic, TopicId};
use engine::repository::sets::{ExistingSetRepository, SetUpdate};
use engine::repository::topics::{ExistingTopicRepository, TopicUpdate};
use engine::repository::{
    EntitiesRepository, IdentifiersRepository, SetsRepository, TopicsRepository,
};
use engine::search_filters::{SetSearchCriteria, TopicSearchCriteria};
use error_stack::{Report, ResultExt};
use tokio_postgres::types::Type;
use tokio_postgres::{Client, Config, NoTls, Statement};
use tracing::error;

pub struct ConnectionDetails {
    pub user: String,
    pub password: String,
    pub database: String,
    pub port: u16,
}

pub type RepoInitResult = std::result::Result<TopicRepo, Report<RepoInitErr>>;

const TOPIC_EXISTS: &str = "select 1 from topics where topic_id = $1";

#[derive(Debug, thiserror::Error)]
#[error("failed to intialize postgres repository")]
pub struct RepoInitErr;

pub async fn init(
    runtime: tokio::runtime::Handle,
    connection_details: ConnectionDetails,
) -> RepoInitResult {
    let (client, connection) = Config::new()
        .user(connection_details.user)
        .password(connection_details.password)
        .dbname(connection_details.database)
        .port(connection_details.port)
        .connect(NoTls)
        .await
        .change_context(RepoInitErr)?;

    runtime.spawn(async move {
        if let Err(e) = connection.await {
            error!("postgres connection error: {e:?}");
        }
    });

    let exists_statement = client
        .prepare_typed(TOPIC_EXISTS, &[Type::UUID])
        .await
        .change_context(RepoInitErr)?;

    Ok(TopicRepo {
        client: Arc::new(client),
        exists_statement,
    })
}

pub struct TopicRepo {
    client: Arc<Client>,
    exists_statement: Statement,
}

impl TopicsRepository for TopicRepo {
    type ExistingTopic = ExistingTopicRepo;

    async fn expect_existing(
        &self,
        topic_id: TopicId,
    ) -> RepoResult<Option<Self::ExistingTopic>, TopicRepoError> {
        Ok(self
            .client
            .query_opt(&self.exists_statement, &[&topic_id.0])
            .await
            .change_context(TopicRepoError::Exists)?
            .map(|_| ExistingTopicRepo(Arc::clone(&self.client))))
    }

    async fn find(&self, topic_id: TopicId) -> RepoResult<Option<Topic>, TopicRepoError> {
        todo!()
    }

    async fn create(
        &self,
        name: String,
        description: Option<String>,
    ) -> RepoResult<Topic, TopicRepoError> {
        todo!()
    }

    async fn search(
        &self,
        topic_search_criteria: TopicSearchCriteria,
    ) -> RepoResult<Vec<Topic>, TopicRepoError> {
        todo!()
    }
}

pub struct ExistingTopicRepo(Arc<Client>); // TODO postgres pool

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

    async fn expect_existing(
        &self,
        set: SetId,
    ) -> RepoResult<Option<Self::ExistingSet>, SetRepoError> {
        todo!()
    }

    async fn find(&self, set_id: SetId) -> RepoResult<Option<Set>, SetRepoError> {
        todo!()
    }

    async fn create(
        &self,
        name: String,
        initial_entity_payloads: Vec<serde_json::value::Value>,
    ) -> RepoResult<Set, SetRepoError> {
        todo!()
    }

    async fn search(
        &self,
        set_search_criteria: SetSearchCriteria,
    ) -> RepoResult<Vec<Set>, SetRepoError> {
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
