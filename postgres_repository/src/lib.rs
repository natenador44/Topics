use std::sync::Arc;

use crate::connection::DbConnection;
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

mod connection;
mod migration;

pub struct ConnectionDetails {
    pub user: String,
    pub password: String,
    pub database: String,
    pub port: u16,
}

pub type RepoInitResult<T> = Result<T, Report<RepoInitErr>>;

#[derive(Debug, thiserror::Error)]
#[error("failed to initialize postgres repository")]
pub struct RepoInitErr;

pub async fn init(
    runtime: tokio::runtime::Handle,
    connection_details: ConnectionDetails,
) -> RepoInitResult<TopicRepo> {
    let (mut client, connection) = Config::new()
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

    migration::run(&mut client).await?;

    Ok(TopicRepo {
        connection: DbConnection::new(client).await?,
    })
}

pub struct TopicRepo {
    connection: DbConnection,
}

impl TopicsRepository for TopicRepo {
    type ExistingTopic = ExistingTopicRepo;

    async fn expect_existing(
        &self,
        topic_id: TopicId,
    ) -> RepoResult<Option<Self::ExistingTopic>, TopicRepoError> {
        Ok(self
            .connection
            .client
            .query_opt(&self.connection.statements.topics.exists, &[&topic_id.0])
            .await
            .change_context(TopicRepoError::Exists)?
            .map(|_| ExistingTopicRepo(self.connection.clone())))
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

pub struct ExistingTopicRepo(DbConnection); // TODO postgres pool

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
