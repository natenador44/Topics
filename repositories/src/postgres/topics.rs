use crate::postgres::insert_many::{InsertMany, InsertManyBuilder, value_set};
use crate::postgres::statements::TopicStatements;
use crate::postgres::{RepoInitErr, sanitize_pagination};
use deadpool_postgres::{Object, Pool};
use error_stack::{Report, ResultExt};
use optional_field::Field;
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;
use tokio_postgres::types::ToSql;
use tokio_stream::StreamExt;
use topics_core::TopicRepository;
use topics_core::list_filter::TopicListCriteria;
use topics_core::model::{NewTopic, PatchTopic, Topic};
use topics_core::result::{CreateErrorType, OptRepoResult, RepoResult, TopicRepoError};
use tracing::warn;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Eq, Clone, Copy, Hash)]
#[serde(transparent)]
pub struct TopicId(pub Uuid);
impl Default for TopicId {
    fn default() -> Self {
        Self::new()
    }
}
impl TopicId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
    pub fn new_with(id: Uuid) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone)]
pub struct TopicRepo {
    pool: Pool,
    statements: TopicStatements,
}

impl TopicRepo {
    pub async fn new(pool: Pool) -> Result<Self, Report<RepoInitErr>> {
        let mut handle = pool.get().await.change_context(RepoInitErr::topics())?;

        let client = &mut **handle;

        Ok(Self {
            statements: TopicStatements::prepare(client)
                .await
                .change_context(RepoInitErr::topics())?,
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
        let pagination = sanitize_pagination(&list_criteria, TopicRepoError::List)?;

        let client = self.client(TopicRepoError::List).await?;

        let topics = client
            .query_raw(
                &self.statements.list,
                &[&pagination.page, &pagination.page_size],
            )
            .await
            .change_context(TopicRepoError::List)?
            .map(|r| r.map(row_to_topic))
            .collect::<Result<_, _>>()
            .await;

        topics.change_context(TopicRepoError::List)
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
        new_topics: Vec<NewTopic>,
    ) -> RepoResult<Vec<RepoResult<Topic<Self::TopicId>>>> {
        let Some(insert) = generate_create_many_insert(new_topics) else {
            warn!("no topic requests sent to data layer, not creating any new topics");
            return Ok(vec![]);
        };

        let client = self
            .client(TopicRepoError::Create(CreateErrorType::DbError))
            .await?;

        let topics = client
            .query_raw(&insert.query, insert.params())
            .await
            .change_context(TopicRepoError::Create(CreateErrorType::DbError))?
            .map(|r| {
                r.map(row_to_topic)
                    .change_context(TopicRepoError::Create(CreateErrorType::DbError))
            })
            .collect()
            .await;

        Ok(topics)
    }

    async fn patch(
        &self,
        id: Self::TopicId,
        patch: PatchTopic,
    ) -> OptRepoResult<Topic<Self::TopicId>> {
        let (stmt, params) = match (&patch.name, &patch.description) {
            (Some(name), Field::Present(description)) => (
                &self.statements.patch_name_desc,
                &[
                    name as &(dyn ToSql + Sync),
                    description as &(dyn ToSql + Sync),
                    &id.0 as &(dyn ToSql + Sync),
                ] as &[&(dyn ToSql + Sync)],
            ),
            (Some(name), Field::Missing) => (
                &self.statements.patch_name,
                &[name as &(dyn ToSql + Sync), &id.0 as &(dyn ToSql + Sync)]
                    as &[&(dyn ToSql + Sync)],
            ),
            (None, Field::Present(description)) => (
                &self.statements.patch_desc,
                &[
                    description as &(dyn ToSql + Sync),
                    &id.0 as &(dyn ToSql + Sync),
                ] as &[&(dyn ToSql + Sync)],
            ),
            (None, Field::Missing) => {
                warn!("no topic patch fields specified, returning existing topic");
                (
                    &self.statements.get,
                    &[&id.0 as &(dyn ToSql + Sync)] as &[&(dyn ToSql + Sync)],
                )
            }
        };

        let topic = self
            .client(TopicRepoError::Patch)
            .await?
            .query_opt(stmt, params)
            .await
            .change_context(TopicRepoError::Patch)?
            .map(row_to_topic);

        Ok(topic)
    }

    async fn delete(&self, id: Self::TopicId) -> OptRepoResult<()> {
        let rows_deleted = self
            .client(TopicRepoError::Delete)
            .await?
            .execute(&self.statements.delete, &[&id.0])
            .await
            .change_context(TopicRepoError::Delete)?;

        Ok((rows_deleted > 0).then_some(()))
    }
}

fn generate_create_many_insert(new_topics: Vec<NewTopic>) -> Option<InsertMany> {
    let mut new_topic_iter = new_topics.into_iter();

    let first = new_topic_iter.next()?;
    let mut builder = InsertManyBuilder::new(
        "topics",
        ["id", "name", "description"],
        value_set![TopicId::new().0 => Uuid, first.name => String, first.description => Option<String>],
    );

    for new_topic in new_topic_iter {
        builder.add_value_set(value_set![TopicId::new().0 => Uuid, new_topic.name => String, new_topic.description => Option<String>]);
    }

    builder.returning(&["id", "name", "description", "created", "updated"]);

    Some(builder.build())
}
