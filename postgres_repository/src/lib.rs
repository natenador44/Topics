use crate::connection::DbConnection;
use engine::error::{RepoResult, SetRepoError, TopicRepoError};
use engine::models::{Set, SetId, Topic, TopicId};
use engine::repository::sets::{ExistingSetRepository, SetUpdate};
use engine::repository::topics::{ExistingTopicRepository, TopicUpdate};
use engine::repository::{
    EntitiesRepository, IdentifiersRepository, SetsRepository, TopicsRepository,
};
use engine::search_filters::{SetSearchCriteria, TopicFilter, TopicSearchCriteria};
use error_stack::{Report, ResultExt};
use optional_field::Field;
use tokio_postgres::types::ToSql;
use tokio_postgres::{NoTls, Row, connect};
use tokio_stream::StreamExt;
use tracing::{debug, error};

mod connection;
mod migration;

pub enum ConnectionDetails {
    Url(String),
    Individual {
        user: String,
        password: String,
        database: String,
        port: u16,
        host: String,
    },
}

pub type RepoInitResult<T> = Result<T, Report<RepoInitErr>>;

#[derive(Debug, thiserror::Error)]
#[error("failed to initialize postgres repository")]
pub struct RepoInitErr;

pub async fn init(
    runtime: tokio::runtime::Handle,
    connection_details: ConnectionDetails,
) -> RepoInitResult<TopicRepo> {
    let (mut client, connection) = match connection_details {
        ConnectionDetails::Url(url) => connect(&url, NoTls).await.change_context(RepoInitErr)?,
        ConnectionDetails::Individual { .. } => {
            todo!("individual connection components not supported yet")
        }
    };

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

#[derive(Debug, Clone)]
pub struct TopicRepo {
    connection: DbConnection,
}

fn row_to_topic(row: Row) -> Topic {
    Topic {
        id: TopicId(row.get("id")),
        name: row.get("name"),
        description: row.get("description"),
        created: row.get("created"),
        updated: row.get("updated"),
    }
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
            .map(|_| ExistingTopicRepo {
                topic_id,
                conn: self.connection.clone(),
            }))
    }

    async fn find(&self, topic_id: TopicId) -> RepoResult<Option<Topic>, TopicRepoError> {
        let result = self
            .connection
            .client
            .query_opt(&self.connection.statements.topics.find, &[&topic_id.0])
            .await
            .change_context(TopicRepoError::Get)?;

        Ok(result.map(row_to_topic))
    }

    async fn create(
        &self,
        name: String,
        description: Option<String>,
    ) -> RepoResult<Topic, TopicRepoError> {
        let id = TopicId::new();
        let row = self
            .connection
            .client
            .query_one(
                &self.connection.statements.topics.create,
                &[&id.0, &name, &description],
            )
            .await
            .change_context(TopicRepoError::Create)?;

        // may eventually grab id, name, and description from the insert call
        // this prevents extra memory allocation though
        Ok(Topic {
            id,
            name,
            description,
            created: row.get("created"),
            updated: None,
        })
    }

    async fn search(
        &self,
        topic_search_criteria: TopicSearchCriteria,
    ) -> RepoResult<Vec<Topic>, TopicRepoError> {
        let client = &self.connection.client;
        let statements = &self.connection.statements;
        let page = topic_search_criteria.page().saturating_sub(1); // assuming users send '1' to specify the first page... we'll see if this sticks, not sure what standard is
        let page_size = topic_search_criteria.page_size();

        let result = match topic_search_criteria.filters() {
            Some(
                [TopicFilter::Name(name), TopicFilter::Description(desc)]
                | [TopicFilter::Description(desc), TopicFilter::Name(name)],
            ) => client
                .query_raw(
                    &statements.topics.name_desc_search,
                    slice_iter(&[name, desc, &page, &page_size]),
                )
                .await
                .change_context(TopicRepoError::Search)?,
            Some([TopicFilter::Name(name)]) => client
                .query_raw(
                    &statements.topics.name_search,
                    slice_iter(&[name, &page, &page_size]),
                )
                .await
                .change_context(TopicRepoError::Search)?,
            Some([TopicFilter::Description(desc)]) => client
                .query_raw(
                    &statements.topics.desc_search,
                    slice_iter(&[desc, &page, &page_size]),
                )
                .await
                .change_context(TopicRepoError::Search)?,
            Some(_) => unreachable!("currently only two topic filters exist"),
            None => client
                .query_raw(
                    &statements.topics.full_search,
                    slice_iter(&[&page, &page_size]),
                )
                .await
                .change_context(TopicRepoError::Search)?,
        };

        result
            .map(|r| r.map(row_to_topic).change_context(TopicRepoError::Search))
            .collect()
            .await
    }
}

pub struct ExistingTopicRepo {
    conn: DbConnection,
    topic_id: TopicId,
}

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
        self.conn
            .client
            .execute(&self.conn.statements.topics.delete, &[&self.topic_id.0])
            .await
            .change_context(TopicRepoError::Delete)?;
        Ok(())
    }

    async fn update(&self, topic: TopicUpdate) -> RepoResult<Topic, TopicRepoError> {
        let row = match (topic.name, topic.description) {
            (Field::Present(n), Field::Present(d)) => self
                .conn
                .client
                .query_one(
                    &self.conn.statements.topics.update_name_desc,
                    &[&n, &d, &self.topic_id.0],
                )
                .await
                .change_context(TopicRepoError::Update)?,
            (Field::Present(n), Field::Missing) => self
                .conn
                .client
                .query_one(
                    &self.conn.statements.topics.update_name,
                    &[&n, &self.topic_id.0],
                )
                .await
                .change_context(TopicRepoError::Update)?,
            (Field::Missing, Field::Present(d)) => self
                .conn
                .client
                .query_one(
                    &self.conn.statements.topics.update_desc,
                    &[&d, &self.topic_id.0],
                )
                .await
                .change_context(TopicRepoError::Update)?,
            _ => {
                debug!("no changes requested, only reading topic");
                self.conn
                    .client
                    .query_one(&self.conn.statements.topics.find, &[&self.topic_id.0])
                    .await
                    .change_context(TopicRepoError::Search)?
            }
        };

        Ok(row_to_topic(row))
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

fn slice_iter<'a>(
    s: &'a [&'a (dyn ToSql + Sync)],
) -> impl ExactSizeIterator<Item = &'a dyn ToSql> + 'a {
    s.iter().map(|s| *s as _)
}
