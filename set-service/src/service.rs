use crate::error::SetServiceError;
use crate::model::Set;
use crate::repository::{NewSet, SetPatch, SetRepo};
use crate::{OptServiceResult, ServiceResult};
use engine::Pagination;
use engine::id::{SetId, TopicId};
use error_stack::ResultExt;
use optional_field::Field;
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct SetService {
    repo: SetRepo,
}

impl SetService {
    pub fn new(repo: SetRepo) -> SetService {
        SetService { repo }
    }

    #[instrument(skip_all, name = "service#get")]
    pub async fn get(&self, id: SetId) -> OptServiceResult<Set> {
        self.repo.get(id).await.change_context(SetServiceError)
    }

    pub async fn list(&self, pagination: Pagination) -> ServiceResult<Vec<Set>> {
        self.repo
            .list(pagination)
            .await
            .change_context(SetServiceError)
    }

    #[instrument(skip_all, name = "service#create")]
    pub async fn create(
        &self,
        topic_id: TopicId,
        name: String,
        description: Option<String>,
    ) -> ServiceResult<Set> {
        self.repo
            .create(NewSet::new(topic_id, name, description))
            .await
            .change_context(SetServiceError)
    }

    #[instrument(skip_all, name = "service#delete")]
    pub async fn delete(&self, set_id: SetId) -> ServiceResult<Option<()>> {
        self.repo
            .delete(set_id)
            .await
            .change_context(SetServiceError)
    }

    #[instrument(skip_all, name = "service#update")]
    pub async fn patch(
        &self,
        set_id: SetId,
        name: Option<String>,
        description: Field<String>,
    ) -> OptServiceResult<Set> {
        self.repo
            .patch(set_id, SetPatch::new(name, description))
            .await
            .change_context(SetServiceError)
    }
}
