use crate::error::TopicServiceError;
use crate::metrics;
use crate::{OptServiceResult, ServiceResult};
use error_stack::ResultExt;
use optional_field::Field;
use topics_core::list_filter::TopicListCriteria;
use topics_core::model::{NewTopic, PatchTopic, Topic};
use topics_core::{CreateManyFailReason, CreateManyTopicStatus, TopicEngine, TopicRepository};
use tracing::{debug, error, instrument};

pub struct TopicCreation {
    name: String,
    description: Option<String>,
}

impl TopicCreation {
    pub fn new(name: String, description: Option<String>) -> Self {
        Self { name, description }
    }
}

pub struct CreateManyTopic {
    name: Field<String>,
    description: Field<String>,
}

impl CreateManyTopic {
    pub fn new(name: Field<String>, description: Field<String>) -> CreateManyTopic {
        Self { name, description }
    }
}

#[derive(Debug, Clone)]
pub struct TopicService<T> {
    engine: T,
}

fn initial_bulk_create_outcome<T>(topic: CreateManyTopic) -> CreateManyTopicStatus<T> {
    match topic.name {
        Field::Present(Some(n)) => CreateManyTopicStatus::Pending {
            name: n,
            description: topic.description.unwrap_present_or(None),
        },
        Field::Present(None) | Field::Missing => CreateManyTopicStatus::Fail {
            topic_name: None,
            topic_description: topic.description.unwrap_present_or(None),
            reason: CreateManyFailReason::MissingName,
        },
    }
}

impl<T> TopicService<T>
where
    T: TopicEngine,
{
    pub fn new(engine: T) -> Self {
        TopicService { engine }
    }

    #[instrument(skip_all, name = "service#get")]
    pub async fn get(&self, id: T::TopicId) -> OptServiceResult<Topic<T::TopicId>> {
        let topic = self
            .engine
            .repo()
            .get(id)
            .await
            .change_context(TopicServiceError)?;

        if topic.is_some() {
            debug!("topic {id:?} found!");
            metrics::increment_topics_retrieved();
        }

        Ok(topic)
    }

    pub async fn list(
        &self,
        list_criteria: TopicListCriteria,
    ) -> ServiceResult<Vec<Topic<T::TopicId>>> {
        let topics = self
            .engine
            .repo()
            .list(list_criteria)
            .await
            .change_context(TopicServiceError)?;

        debug!("{} topics found", topics.len());
        metrics::increment_topics_retrieved_by(topics.len());
        Ok(topics)
    }

    #[instrument(skip_all, name = "service#create")]
    pub async fn create(&self, topic: TopicCreation) -> ServiceResult<Topic<T::TopicId>> {
        let topic = self
            .engine
            .repo()
            .create(NewTopic::new(topic.name, topic.description))
            .await
            .change_context(TopicServiceError)?;

        debug!("created topic");
        metrics::increment_topics_created();
        Ok(topic)
    }

    #[instrument(skip_all, name = "service#create_many")]
    pub async fn create_many<I>(
        &self,
        topics: I,
    ) -> ServiceResult<Vec<CreateManyTopicStatus<T::TopicId>>>
    where
        I: Iterator<Item = CreateManyTopic> + Send + Sync + 'static,
    {
        let mut statuses = Vec::new();
        let mut pending_topics = Vec::new();

        let mut status_indexes = Vec::new();

        for (i, topic_req) in topics.enumerate() {
            let status = initial_bulk_create_outcome(topic_req);

            if let CreateManyTopicStatus::Pending { name, description } = &status {
                pending_topics.push(NewTopic::new(name.clone(), description.clone()));
                status_indexes.push(i);
            }

            statuses.push(status);
        }

        if pending_topics.is_empty() {
            return Ok(statuses);
        }

        let new_topic_results = self
            .engine
            .repo()
            .create_many(pending_topics)
            .await
            .change_context(TopicServiceError)?;

        let mut created_topics_count = 0;

        for (i, topic_result) in new_topic_results.into_iter().enumerate() {
            let status_idx = status_indexes[i];
            let status = &mut statuses[status_idx];
            match topic_result {
                Ok(topic) => {
                    created_topics_count += 1;
                    *status = CreateManyTopicStatus::Success(topic);
                }
                Err(e) => {
                    error!("Topic request (idx: {status_idx}) failed with error '{e}'");
                    if let CreateManyTopicStatus::Pending { name, description } = status {
                        *status = CreateManyTopicStatus::Fail {
                            topic_name: Some(std::mem::take(name)),
                            topic_description: description.take(),
                            reason: CreateManyFailReason::ServiceError,
                        };
                    } else {
                        unreachable!("Topic result respective status should only be 'Pending'");
                    }
                }
            }
        }

        debug!(
            "created {} out of {} requested topics",
            created_topics_count,
            statuses.len(),
        );
        metrics::increment_topics_created_by(created_topics_count);
        Ok(statuses)
    }

    #[instrument(skip_all, name = "service#delete")]
    pub async fn delete(&self, topic_id: T::TopicId) -> ServiceResult<Option<()>> {
        let deleted = self
            .engine
            .repo()
            .delete(topic_id)
            .await
            .change_context(TopicServiceError)?;

        if deleted.is_some() {
            debug!("deleted topic {topic_id:?}");
            metrics::increment_topics_deleted();
        }

        Ok(deleted)
    }

    #[instrument(skip_all, name = "service#update")]
    pub async fn patch(
        &self,
        topic_id: T::TopicId,
        name: Field<String>,
        description: Field<String>,
    ) -> ServiceResult<PatchOutcome<T::TopicId>> {
        let name = match name {
            Field::Present(Some(n)) => Some(n),
            Field::Missing => None,
            Field::Present(None) => {
                // name cannot be null
                return Ok(PatchOutcome::InvalidName);
            }
        };

        let topic = self
            .engine
            .repo()
            .patch(topic_id, PatchTopic::new(name, description))
            .await
            .change_context(TopicServiceError)?;

        if topic.is_some() {
            debug!("patched {topic_id:?}");
            metrics::increment_topics_patched();
        }
        Ok(topic
            .map(PatchOutcome::Success)
            .unwrap_or(PatchOutcome::NotFound))
    }
}

pub enum PatchOutcome<T> {
    Success(Topic<T>),
    InvalidName,
    NotFound,
}
