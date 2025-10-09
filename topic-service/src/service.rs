use error_stack::ResultExt;
use optional_field::Field;
use engine::Pagination;
use tracing::instrument;
use engine::id::TopicId;
use crate::{OptServiceResult, ServiceResult};
use crate::error::TopicServiceError;
use crate::model::{Topic};
use crate::repository::{NewTopic, TopicPatch, TopicRepo};

#[derive(Debug, Clone)]
pub struct TopicService {
    repo: TopicRepo,
}

impl TopicService
{
    pub fn new(repo: TopicRepo) -> TopicService {
        TopicService { repo }
    }

    #[instrument(skip_all, name = "service#get")]
    pub async fn get(
        &self,
        id: TopicId,
    ) -> OptServiceResult<Topic> {
        self.repo
            .get(id).await
            .change_context(TopicServiceError)
    }

    pub async fn list(
        &self,
        pagination: Pagination,
    ) -> ServiceResult<Vec<Topic>> {
        self.repo
            .list(pagination)
            .await
            .change_context(TopicServiceError)
    }

    #[instrument(skip_all, name = "service#create")]
    pub async fn create(
        &self,
        name: String,
        description: Option<String>
    ) -> ServiceResult<Topic> {
        self.repo
            .create(NewTopic::new(name, description))
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
        description: Field<String>
    ) -> OptServiceResult<Topic> {
        self.repo
            .patch(topic_id, TopicPatch::new(name, description))
            .await
            .change_context(TopicServiceError)
    }
}
