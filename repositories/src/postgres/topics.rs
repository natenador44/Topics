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
use tracing::{error, warn};
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

    // TODO I really don't like how I have this... maybe it's better to allocate more memory to
    // prevent so much complexity
    async fn create_many(
        &self,
        mut create_new_topic_statuses: Vec<CreateManyTopicStatus<Self::TopicId>>,
    ) -> RepoResult<Vec<CreateManyTopicStatus<Self::TopicId>>> {
        let Some(insert) = generate_create_many_insert(&create_new_topic_statuses) else {
            warn!(
                "No statuses in pending status, not creating topics in db. This means the request had errors previously"
            );
            return Ok(create_new_topic_statuses);
        };

        let client = self
            .client(TopicRepoError::Create(CreateErrorType::DbError))
            .await?;

        let new_topics = client
            .query_raw(&insert.query, insert.params())
            .await
            .change_context(TopicRepoError::Create(CreateErrorType::DbError))?
            .map(|r| r.map(row_to_topic))
            .collect::<Vec<_>>()
            .await;

        // go through created topics and match with index (0 == 0, 1, == 1), then update status using that index

        for (i, topic_result) in new_topics.into_iter().enumerate() {
            let status_idx = insert.status_indexes_involved[i];

            let Some(status) = create_new_topic_statuses.get_mut(status_idx) else {
                return Err(TopicRepoError::Create(CreateErrorType::MatchFailure))
                    .attach_with(|| format!("status idx {status_idx} did not exist"))?;
            };

            *status = match topic_result {
                Ok(topic) => CreateManyTopicStatus::Success(topic),
                Err(e) => {
                    error!("failed to create topic (status idx: {status_idx}): {e}");
                    let CreateManyTopicStatus::Pending { name, description } = status else {
                        return Err(TopicRepoError::Create(CreateErrorType::MatchFailure))
                            .attach_with(|| format!("status to update was not pending (idx: {status_idx}). this is a bug"))?;
                    };

                    CreateManyTopicStatus::Fail {
                        topic_name: Some(std::mem::take(name)),
                        topic_description: description.take(),
                        reason: CreateManyFailReason::ServiceError,
                    }
                }
            };
        }

        Ok(create_new_topic_statuses)
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

struct CreateManyInsert {
    status_indexes_involved: Vec<usize>,
    params: Vec<(TopicId, String, Option<String>)>,
    query: String,
}

impl CreateManyInsert {
    fn builder() -> CreateManyInsertBuilder {
        CreateManyInsertBuilder {
            status_indexes_involved: Vec::default(),
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
    status_indexes_involved: Vec<usize>,
    params: Vec<(TopicId, String, Option<String>)>,
}

impl CreateManyInsertBuilder {
    fn add_new(&mut self, status_index: usize, name: &str, description: Option<&str>) {
        self.status_indexes_involved.push(status_index);
        self.params.push((
            TopicId::new(),
            name.to_string(),
            description.map(|s| s.to_string()),
        ));
    }

    fn build(self) -> Option<CreateManyInsert> {
        if self.params.is_empty() {
            return None;
        }

        let mut query = String::from("insert into topics (id, name, description) values ");
        let mut param_count = 0;
        for (i, (_)) in self.params.iter().enumerate() {
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
            status_indexes_involved: self.status_indexes_involved,
            params: self.params,
            query,
        })
    }
}

fn generate_create_many_insert(
    statuses: &[CreateManyTopicStatus<TopicId>],
) -> Option<CreateManyInsert> {
    let mut builder = CreateManyInsert::builder();

    for (i, status) in statuses.iter().enumerate() {
        if let CreateManyTopicStatus::Pending { name, description } = status {
            builder.add_new(i, name, description.as_deref());
        }
    }

    builder.build()
}

#[cfg(test)]
mod tests {
    use crate::postgres::topics::{TopicId, generate_create_many_insert};
    use chrono::Utc;
    use topics_core::model::Topic;
    use topics_core::{CreateManyFailReason, CreateManyTopicStatus};

    #[test]
    fn generate_create_many_insert_empty_statuses_returns_none() {
        assert!(generate_create_many_insert(&[]).is_none())
    }

    #[test]
    fn generate_create_many_insert_returns_none_if_no_pending_statuses() {
        let statuses = vec![
            CreateManyTopicStatus::Fail {
                topic_name: None,
                topic_description: None,
                reason: CreateManyFailReason::MissingName,
            },
            CreateManyTopicStatus::Success(Topic::new(
                TopicId::new(),
                "t1".into(),
                None,
                Utc::now(),
                None,
            )),
        ];

        assert!(generate_create_many_insert(&statuses).is_none())
    }

    #[test]
    fn generate_create_many_insert_creates_insert_for_all_pending_statuses() {
        let statuses = vec![
            CreateManyTopicStatus::Fail {
                topic_name: None,
                topic_description: None,
                reason: CreateManyFailReason::MissingName,
            },
            CreateManyTopicStatus::Pending {
                name: "topic2".into(),
                description: Some("desc2".into()),
            },
            CreateManyTopicStatus::Success(Topic::new(
                TopicId::new(),
                "t1".into(),
                None,
                Utc::now(),
                None,
            )),
            CreateManyTopicStatus::Pending {
                name: "topic1".into(),
                description: None,
            },
        ];

        let insert = generate_create_many_insert(&statuses).unwrap();

        assert_eq!(2, insert.params.len());
        assert_eq!("topic2", &insert.params[0].1);
        assert_eq!(Some("desc2"), insert.params[0].2.as_deref());
        assert_eq!("topic1", &insert.params[1].1);
        assert_eq!(None, insert.params[1].2.as_deref());

        let expected_query = "insert into topics (id, name, description) values ($1, $2, $3), ($4, $5, $6) returning id, name, description, created, updated";

        assert_eq!(expected_query, insert.query);
        assert_eq!(vec![1, 3], insert.status_indexes_involved);
    }
}
