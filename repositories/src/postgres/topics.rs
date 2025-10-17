use crate::postgres::statements::Statements;
use error_stack::{IntoReport, Report, ResultExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::task::JoinSet;
use tokio_postgres::{Client, Row, Statement};
use tokio_stream::StreamExt;
use topics_core::list_filter::TopicListCriteria;
use topics_core::model::{NewTopic, PatchTopic, Topic};
use topics_core::result::{CreateErrorType, OptRepoResult, RepoResult, TopicRepoError};
use topics_core::{CreateManyFailReason, CreateManyTopicStatus, TopicRepository};
use tracing::error;
use utoipa::ToSchema;
use uuid::Uuid;

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
    client: Arc<Client>,
    statements: Statements,
}

impl TopicRepo {
    pub async fn init(connection_details: ConnectionDetails) -> Result<Self, Report<InitErr>> {
        todo!()
    }

    pub async fn new(client: Client) -> Result<Self, Report<InitErr>> {
        Ok(Self {
            statements: Statements::prepare(&client).await.change_context(InitErr)?,
            client: Arc::new(client),
        })
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
        let topic = self
            .client
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

        let topics = self
            .client
            .query_raw(&self.statements.list, &[&page, &page_size])
            .await
            .change_context(TopicRepoError::List)?
            .map(|r| r.map(row_to_topic))
            .collect::<Result<_, _>>()
            .await;

        Ok(topics.change_context(TopicRepoError::List)?)
    }

    async fn create(&self, new_topic: NewTopic) -> RepoResult<Topic<Self::TopicId>> {
        self.client
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
        let mut creates = JoinSet::new();

        for (i, topic) in topics.iter().enumerate() {
            if let CreateManyTopicStatus::Pending { name, description } = topic {
                let new_topic = NewTopic::new(name.clone(), description.clone());
                let c = Arc::clone(&self.client);
                let s = self.statements.create.clone();

                creates.spawn(async move { (i, create(c, s, new_topic).await) });
            }
        }

        while let Some(result) = creates.join_next().await {
            let Ok((i, topic_result)) = result else {
                return Err(TopicRepoError::Create(CreateErrorType::MatchFailure))
                    .attach("failed to join on create topic task");
            };

            let status = topics
                .get_mut(i)
                .ok_or(TopicRepoError::Create(CreateErrorType::MatchFailure))?;

            match topic_result {
                Ok(topic) => {
                    *status = CreateManyTopicStatus::Success(topic);
                }
                Err(e) => match status {
                    CreateManyTopicStatus::Pending { name, description } => {
                        *status = CreateManyTopicStatus::Fail {
                            topic_name: Some(std::mem::take(name)),
                            topic_description: description.take(),
                            reason: CreateManyFailReason::ServiceError,
                        }
                    }
                    _ => {}
                },
            }
        }

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
    client: Arc<Client>,
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
