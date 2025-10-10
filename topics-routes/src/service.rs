use crate::error::TopicServiceError;
use crate::{OptServiceResult, ServiceResult};
use error_stack::ResultExt;
use optional_field::Field;
use topics_core::{NewTopic, PatchTopic, Topic, TopicEngine, TopicListCriteria, TopicRepository};
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct TopicService<T> {
    engine: T,
}

impl<T> TopicService<T>
where
    T: TopicEngine,
    T::Repo: TopicRepository,
{
    pub fn new(engine: T) -> Self {
        TopicService { engine }
    }

    #[instrument(skip_all, name = "service#get")]
    pub async fn get(&self, id: T::TopicId) -> OptServiceResult<Topic<T::TopicId>> {
        self.engine
            .repo()
            .get(id)
            .await
            .change_context(TopicServiceError)
    }

    pub async fn list(
        &self,
        list_criteria: TopicListCriteria,
    ) -> ServiceResult<Vec<Topic<T::TopicId>>> {
        self.engine
            .repo()
            .list(list_criteria)
            .await
            .change_context(TopicServiceError)
    }

    #[instrument(skip_all, name = "service#create")]
    pub async fn create(
        &self,
        name: String,
        description: Option<String>,
    ) -> ServiceResult<Topic<T::TopicId>> {
        self.engine
            .repo()
            .create(NewTopic::new(name, description))
            .await
            .change_context(TopicServiceError)
    }

    #[instrument(skip_all, name = "service#delete")]
    pub async fn delete(&self, topic_id: T::TopicId) -> ServiceResult<Option<()>> {
        self.engine
            .repo()
            .delete(topic_id)
            .await
            .change_context(TopicServiceError)
    }

    #[instrument(skip_all, name = "service#update")]
    pub async fn patch(
        &self,
        topic_id: T::TopicId,
        name: Option<String>,
        description: Field<String>,
    ) -> OptServiceResult<Topic<T::TopicId>> {
        self.engine
            .repo()
            .patch(topic_id, PatchTopic::new(name, description))
            .await
            .change_context(TopicServiceError)
    }
}
