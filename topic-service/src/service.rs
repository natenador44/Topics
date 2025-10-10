use crate::error::TopicServiceError;
use crate::filter::TopicListCriteria;
use crate::model::Topic;
use crate::repository::{NewTopicRequest, TopicPatch, TopicRepo};
use crate::{OptServiceResult, ServiceResult};
use engine::Pagination;
use engine::id::TopicId;
use engine::list_criteria::ListCriteria;
use error_stack::ResultExt;
use optional_field::Field;
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct TopicService {
    repo: TopicRepo,
}

impl TopicService {
    pub fn new(repo: TopicRepo) -> TopicService {
        TopicService { repo }
    }

    #[instrument(skip_all, name = "service#get")]
    pub async fn get(&self, id: TopicId) -> OptServiceResult<Topic> {
        self.repo.get(id).await.change_context(TopicServiceError)
    }

    pub async fn list(&self, list_criteria: TopicListCriteria) -> ServiceResult<Vec<Topic>> {
        self.repo
            .list(list_criteria)
            .await
            .change_context(TopicServiceError)
    }

    #[instrument(skip_all, name = "service#create")]
    pub async fn create(&self, name: String, description: Option<String>) -> ServiceResult<Topic> {
        self.repo
            .create(NewTopicRequest::new(name, description))
            .await
            .change_context(TopicServiceError)
    }

    #[instrument(skip_all, name = "service#delete")]
    pub async fn delete(&self, topic_id: TopicId) -> ServiceResult<Option<()>> {
        self.repo
            .delete(topic_id)
            .await
            .change_context(TopicServiceError)
    }

    #[instrument(skip_all, name = "service#update")]
    pub async fn patch(
        &self,
        topic_id: TopicId,
        name: Option<String>,
        description: Field<String>,
    ) -> OptServiceResult<Topic> {
        self.repo
            .patch(topic_id, TopicPatch::new(name, description))
            .await
            .change_context(TopicServiceError)
    }
}
