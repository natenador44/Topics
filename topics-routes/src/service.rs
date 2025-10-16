use crate::error::TopicServiceError;
use crate::metrics;
use crate::{OptServiceResult, ServiceResult};
use error_stack::ResultExt;
use optional_field::Field;
use topics_core::list_filter::TopicListCriteria;
use topics_core::model::{NewTopic, PatchTopic, Topic};
use topics_core::{CreateManyFailReason, CreateManyTopicStatus, TopicEngine, TopicRepository};
use tracing::{info, instrument};

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

        info!("created topics");
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
        let initial_outcomes = topics.map(initial_bulk_create_outcome).collect::<Vec<_>>();
        let requested_topics_count = initial_outcomes.len();

        let final_outcomes = self
            .engine
            .repo()
            .create_many(initial_outcomes)
            .await
            .change_context(TopicServiceError)?;

        info!(
            "created {} out of {} requested topics",
            final_outcomes
                .iter()
                .filter(|o| matches!(o, CreateManyTopicStatus::Success(_)))
                .count(),
            requested_topics_count,
        );
        metrics::increment_topics_created_by(final_outcomes.len());
        Ok(final_outcomes)
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
            info!("deleted topic");
            metrics::increment_topics_deleted();
        }

        Ok(deleted)
    }

    #[instrument(skip_all, name = "service#update")]
    pub async fn patch(
        &self,
        topic_id: T::TopicId,
        name: Option<String>,
        description: Field<String>,
    ) -> OptServiceResult<Topic<T::TopicId>> {
        let topic = self
            .engine
            .repo()
            .patch(topic_id, PatchTopic::new(name, description))
            .await
            .change_context(TopicServiceError)?;

        if topic.is_some() {
            metrics::increment_topics_patched();
        }
        Ok(topic)
    }
}
