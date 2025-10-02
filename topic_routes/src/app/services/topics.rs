use crate::app::services::ResourceOutcome;
use crate::error::AppResult;
use crate::error::TopicServiceError;
use engine::models::{Topic, TopicId};
use engine::repository::TopicsRepository;
use engine::repository::topics::{ExistingTopicRepository, TopicUpdate};
use engine::search_filters::{TopicFilter, TopicSearchCriteria};
use engine::{Engine, Pagination};
use error_stack::ResultExt;
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct TopicService<T> {
    engine: T,
}

impl<T: Engine> TopicService<T> {
    pub fn new(engine: T) -> TopicService<T> {
        TopicService { engine }
    }

    #[instrument(skip_all, ret(level = "debug"), name = "service#search")]
    pub async fn search(
        &self,
        search_criteria: TopicSearchCriteria,
    ) -> AppResult<Vec<Topic>, TopicServiceError> {
        let topic_repo = self.engine.topics();
        let page = search_criteria.page();
        let page_size = search_criteria.page_size();

        let topics = topic_repo
            .search(search_criteria)
            .await
            .change_context(TopicServiceError)?;

        Ok(topics)
    }

    #[instrument(skip_all, ret(level = "debug"), name = "service#get_by_id")]
    pub async fn get(&self, topic_id: TopicId) -> AppResult<Option<Topic>, TopicServiceError> {
        let topic = self
            .engine
            .topics()
            .find(topic_id)
            .await
            .change_context(TopicServiceError)?;
        Ok(topic)
    }

    #[instrument(skip_all, ret(level = "debug"), name = "service#create")]
    pub async fn create(
        &self,
        name: String,
        description: Option<String>,
    ) -> AppResult<Topic, TopicServiceError> {
        self.engine
            .topics()
            .create(name, description)
            .await
            .change_context(TopicServiceError)
    }

    #[instrument(skip_all, ret(level = "debug"), name = "service#delete")]
    pub async fn delete(&self, topic_id: TopicId) -> AppResult<ResourceOutcome, TopicServiceError> {
        let Some(topic) = self
            .engine
            .topics()
            .expect_existing(topic_id)
            .await
            .change_context(TopicServiceError)?
        else {
            return Ok(ResourceOutcome::NotFound);
        };

        topic.delete().await.change_context(TopicServiceError)?;
        Ok(ResourceOutcome::Found)
    }

    #[instrument(skip_all, ret(level = "debug"), name = "service#update")]
    pub async fn update(
        &self,
        topic_id: TopicId,
        name: Option<String>,
        description: Option<String>,
    ) -> AppResult<Option<Topic>, TopicServiceError> {
        let Some(topic) = self
            .engine
            .topics()
            .expect_existing(topic_id)
            .await
            .change_context(TopicServiceError)?
        else {
            return Ok(None);
        };

        topic
            .update(TopicUpdate { name, description })
            .await
            .change_context(TopicServiceError)
            .map(Some)
    }
}
