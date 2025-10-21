use crate::postgres::statements::Statements;
use deadpool_postgres::{Manager, ManagerConfig, Object, Pool, RecyclingMethod};
use error_stack::{IntoReport, Report, ResultExt};
use optional_field::Field;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tokio_postgres::types::ToSql;
use tokio_postgres::{Client, Config, NoTls, Row};
use tokio_stream::StreamExt;
use topics_core::TopicRepository;
use topics_core::list_filter::TopicListCriteria;
use topics_core::model::{NewTopic, PatchTopic, Topic};
use topics_core::result::{CreateErrorType, OptRepoResult, RepoResult, TopicRepoError};
use tracing::{error, info, warn};
use utoipa::ToSchema;
use uuid::Uuid;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./src/postgres/migrations/topics");
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Eq, Clone, Copy)]
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

macro_rules! validate_pagination_field {
    ($field_name:literal, $field:expr) => {
        if $field > i64::MAX as u64 {
            return Err(TopicRepoError::List.into_report()).attach_with(|| {
                format!(
                    "{} '{}' is too large and is not supported",
                    $field_name, $field
                )
            });
        } else {
            $field as i64
        }
    };

    ($field_name:literal, $field:expr => $map:expr) => {
        if $field > i64::MAX as u64 {
            return Err(TopicRepoError::List.into_report()).attach_with(|| {
                format!(
                    "{} '{}' is too large and is not supported",
                    $field_name, $field
                )
            });
        } else {
            $map as i64
        }
    };
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
        let page = validate_pagination_field!("page", list_criteria.page() => list_criteria.page().saturating_sub(1));
        let page_size = validate_pagination_field!("page_size", list_criteria.page_size());

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
        todo!()
    }
}

struct CreateManyInsert {
    params: Vec<(TopicId, String, Option<String>)>,
    query: String,
}

impl CreateManyInsert {
    fn builder() -> CreateManyInsertBuilder {
        CreateManyInsertBuilder {
            params: Vec::default(),
        }
    }

    fn params(&self) -> Vec<&(dyn ToSql + Sync)> {
        let mut p_ref = Vec::with_capacity(self.params.len());

        for (id, name, desc) in &self.params {
            p_ref.push(&id.0 as &(dyn ToSql + Sync));
            p_ref.push(name as &(dyn ToSql + Sync));
            p_ref.push(desc as &(dyn ToSql + Sync));
        }

        p_ref
    }
}

struct CreateManyInsertBuilder {
    params: Vec<(TopicId, String, Option<String>)>,
}

impl CreateManyInsertBuilder {
    fn add_new(&mut self, name: String, description: Option<String>) {
        self.params.push((TopicId::new(), name, description));
    }

    fn build(self) -> Option<CreateManyInsert> {
        if self.params.is_empty() {
            return None;
        }

        let mut query = String::from("insert into topics (id, name, description) values ");
        let mut param_count = 0;
        for (i, _) in self.params.iter().enumerate() {
            query += &format!(
                "(${}, ${}, ${})",
                param_count + 1,
                param_count + 2,
                param_count + 3
            );

            if i < self.params.len() - 1 {
                query += ", ";
            }
            param_count += 3;
        }

        query += " returning id, name, description, created, updated";
        Some(CreateManyInsert {
            params: self.params,
            query,
        })
    }
}

fn generate_create_many_insert(new_topics: Vec<NewTopic>) -> Option<CreateManyInsert> {
    let mut builder = CreateManyInsert::builder();

    for new_topic in new_topics {
        builder.add_new(new_topic.name, new_topic.description)
    }

    builder.build()
}

#[cfg(test)]
mod tests {
    use crate::postgres::topics::{TopicId, generate_create_many_insert};
    use chrono::Utc;
    use topics_core::model::{NewTopic, Topic};
    use topics_core::{CreateManyFailReason, CreateManyTopicStatus};

    #[test]
    fn generate_create_many_insert_empty_list_returns_none() {
        assert!(generate_create_many_insert(vec![]).is_none())
    }

    #[test]
    fn generate_create_many_insert_creates_insert_for_all_pending_statuses() {
        let new_topics = vec![
            NewTopic::new("topic1".into(), Some("topic1 desc".into())),
            NewTopic::new("topic2".into(), Some("topic2 desc".into())),
            NewTopic::new("topic3".into(), Some("topic3 desc".into())),
        ];

        let insert = generate_create_many_insert(new_topics.clone()).unwrap();

        for i in 1..=3 {
            let (_, p_name, p_desc) = &insert.params[i - 1];
            assert_eq!(&new_topics[i - 1].name, p_name);
            assert_eq!(&new_topics[i - 1].description, p_desc);
        }

        let expected_query = "insert into topics (id, name, description) values ($1, $2, $3), ($4, $5, $6), ($7, $8, $9) returning id, name, description, created, updated";

        assert_eq!(expected_query, insert.query);
    }
}
