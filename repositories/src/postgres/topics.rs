use crate::postgres::statements::Statements;
use deadpool_postgres::{Manager, ManagerConfig, Object, Pool, RecyclingMethod};
use error_stack::{IntoReport, Report, ResultExt};
use serde::{Deserialize, Serialize};
use std::ops::DerefMut;
use std::str::FromStr;
use std::sync::Arc;
use tokio::task::JoinSet;
use tokio_postgres::types::ToSql;
use tokio_postgres::{Client, Config, NoTls, Row, Statement};
use tokio_stream::StreamExt;
use topics_core::list_filter::TopicListCriteria;
use topics_core::model::{NewTopic, PatchTopic, Topic};
use topics_core::result::{CreateErrorType, OptRepoResult, RepoResult, TopicRepoError};
use topics_core::{CreateManyFailReason, CreateManyTopicStatus, TopicRepository};
use tracing::error;
use utoipa::ToSchema;
use uuid::Uuid;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./src/postgres/migrations/topics");
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Eq, Clone)]
#[serde(transparent)]
pub struct TopicId(Uuid);
impl TopicId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
    pub fn new_with(id: Uuid) -> Self {
        Self(id)
    }
}

pub enum ConnectionDetails {
    Url(String),
}

#[derive(Debug, thiserror::Error)]
#[error("failed to initialize postgres topics repo")]
pub struct InitErr;

#[derive(Debug, Clone)]
pub struct TopicRepo {
    pub(crate) pool: Pool,
    statements: Statements,
}

async fn run_migrations(client: &mut Client) -> Result<(), Report<InitErr>> {
    embedded::migrations::runner()
        .run_async(client)
        .await
        .change_context(InitErr)?;
    Ok(())
}

impl TopicRepo {
    pub async fn init(connection_details: ConnectionDetails) -> Result<Self, Report<InitErr>> {
        let config = match connection_details {
            ConnectionDetails::Url(url) => Config::from_str(&url).change_context(InitErr)?,
        };

        let mgr_config = ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        };
        let mgr = Manager::from_config(config, NoTls, mgr_config);
        let pool = Pool::builder(mgr).build().change_context(InitErr)?;

        let mut handle = pool.get().await.change_context(InitErr)?;

        let client = &mut *(&mut *handle);

        run_migrations(client).await?;

        Ok(Self {
            statements: Statements::prepare(client).await.change_context(InitErr)?,
            pool,
        })
    }

    pub async fn init_with_pool_size(
        connection_details: ConnectionDetails,
        pool_size: usize,
    ) -> Result<Self, Report<InitErr>> {
        let config = match connection_details {
            ConnectionDetails::Url(url) => Config::from_str(&url).change_context(InitErr)?,
        };

        let mgr_config = ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        };
        let mgr = Manager::from_config(config, NoTls, mgr_config);
        let pool = Pool::builder(mgr)
            .max_size(pool_size)
            .build()
            .change_context(InitErr)?;

        let mut handle = pool.get().await.change_context(InitErr)?;

        let client = &mut *(&mut *handle);

        run_migrations(client).await?;

        Ok(Self {
            statements: Statements::prepare(client).await.change_context(InitErr)?,
            pool,
        })
    }

    async fn client(&self, on_fail: TopicRepoError) -> RepoResult<Object> {
        self.pool.get().await.change_context(on_fail)
    }
}

fn row_to_topic(row: Row) -> Topic<TopicId> {
    Topic::new(
        TopicId(row.get("id")),
        row.get("name"),
        row.get("description"),
        row.get("created"),
        row.get("updated"),
    )
}

impl TopicRepository for TopicRepo {
    type TopicId = TopicId;

    async fn get(&self, id: Self::TopicId) -> OptRepoResult<Topic<Self::TopicId>> {
        let client = self.client(TopicRepoError::Get).await?;

        let topic = client
            .query_opt(&self.statements.get, &[&id.0])
            .await
            .change_context(TopicRepoError::Get)?
            .map(row_to_topic);
        Ok(topic)
    }

    async fn list(
        &self,
        list_criteria: TopicListCriteria,
    ) -> RepoResult<Vec<Topic<Self::TopicId>>> {
        let page = if list_criteria.page() > i64::MAX as u64 {
            return Err(TopicRepoError::List.into_report()).attach_with(|| {
                format!(
                    "page '{}' is too large and is not supported",
                    list_criteria.page()
                )
            });
        } else {
            list_criteria.page() as i64
        };

        let page_size = list_criteria.page_size().min(i64::MAX as u64) as i64;

        let client = self.client(TopicRepoError::List).await?;

        let topics = client
            .query_raw(&self.statements.list, &[&page, &page_size])
            .await
            .change_context(TopicRepoError::List)?
            .map(|r| r.map(row_to_topic))
            .collect::<Result<_, _>>()
            .await;

        Ok(topics.change_context(TopicRepoError::List)?)
    }

    async fn create(&self, new_topic: NewTopic) -> RepoResult<Topic<Self::TopicId>> {
        let client = self
            .client(TopicRepoError::Create(CreateErrorType::DbError))
            .await?;

        client
            .query_one(
                &self.statements.create,
                &[&TopicId::new().0, &new_topic.name, &new_topic.description],
            )
            .await
            .change_context(TopicRepoError::Create(CreateErrorType::DbError))
            .map(row_to_topic)
    }

    async fn create_many(
        &self,
        mut topics: Vec<CreateManyTopicStatus<Self::TopicId>>,
    ) -> RepoResult<Vec<CreateManyTopicStatus<Self::TopicId>>> {
        let mut stmt = String::from("insert into topics (id, name, description) values ");

        let mut idxs = Vec::new();
        let mut params = Vec::new();

        for (i, status) in topics.iter().enumerate() {
            if let CreateManyTopicStatus::Pending { name, description } = status {
                if !idxs.is_empty() {
                    // if we just added a value segment, and we're about to add a new one, we need a comma
                    stmt.push_str(", ");
                }
                let p_count = params.len() * 3;
                stmt.push_str(&format!(
                    "(${}, ${}, ${}) ",
                    p_count + 1,
                    p_count + 2,
                    p_count + 3
                ));
                params.push((TopicId::new(), name.clone(), description.clone()));
                idxs.push(i);
            }
        }

        let mut p_ref = Vec::with_capacity(params.len());

        for (id, name, desc) in &params {
            p_ref.push(&id.0 as &(dyn ToSql + Sync));
            p_ref.push(name as &(dyn ToSql + Sync));
            p_ref.push(desc as &(dyn ToSql + Sync));
        }

        let client = self
            .client(TopicRepoError::Create(CreateErrorType::DbError))
            .await?;

        let new_topics = client
            .query_raw(&stmt, p_ref)
            .await
            .change_context(TopicRepoError::Create(CreateErrorType::DbError))?
            .map(|r| {
                r.map(row_to_topic)
                    .change_context(TopicRepoError::Create(CreateErrorType::DbError))
            })
            .collect::<Result<Vec<_>, _>>()
            .await?;

        // go through created topics and match with index (0 == 0, 1, == 1), then update status using that index

        Ok(topics)
    }

    async fn patch(
        &self,
        id: Self::TopicId,
        patch: PatchTopic,
    ) -> OptRepoResult<Topic<Self::TopicId>> {
        todo!()
    }

    async fn delete(&self, id: Self::TopicId) -> OptRepoResult<()> {
        todo!()
    }
}

async fn create(
    client: Object,
    statement: Statement,
    new_topic: NewTopic,
) -> RepoResult<Topic<TopicId>> {
    client
        .query_one(
            &statement,
            &[&TopicId::new().0, &new_topic.name, &new_topic.description],
        )
        .await
        .change_context(TopicRepoError::Create(CreateErrorType::DbError))
        .map(row_to_topic)
}
