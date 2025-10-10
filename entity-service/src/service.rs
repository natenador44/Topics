use crate::error::EntityServiceError;
use crate::model::Entity;
use crate::repository::{EntityPatch, EntityRepo, NewEntity};
use crate::{OptServiceResult, ServiceResult};
use engine::Pagination;
use engine::id::{EntityId, SetId, TopicId};
use error_stack::ResultExt;
use serde_json::Value;
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct EntityService {
    repo: EntityRepo,
}

impl EntityService {
    pub fn new(repo: EntityRepo) -> EntityService {
        EntityService { repo }
    }

    #[instrument(skip_all, name = "service#get")]
    pub async fn get(&self, id: EntityId) -> OptServiceResult<Entity> {
        self.repo.get(id).await.change_context(EntityServiceError)
    }

    pub async fn list(&self, pagination: Pagination) -> ServiceResult<Vec<Entity>> {
        self.repo
            .list(pagination)
            .await
            .change_context(EntityServiceError)
    }

    #[instrument(skip_all, name = "service#create")]
    pub async fn create(
        &self,
        topic_id: TopicId,
        set_id: SetId,
        payload: Value,
    ) -> ServiceResult<Entity> {
        self.repo
            .create(NewEntity::new(topic_id, set_id, payload))
            .await
            .change_context(EntityServiceError)
    }

    #[instrument(skip_all, name = "service#delete")]
    pub async fn delete(&self, entity_id: EntityId) -> ServiceResult<Option<()>> {
        self.repo
            .delete(entity_id)
            .await
            .change_context(EntityServiceError)
    }

    #[instrument(skip_all, name = "service#update")]
    pub async fn patch(
        &self,
        entity_id: EntityId,
        payload: Option<Value>,
    ) -> OptServiceResult<Entity> {
        self.repo
            .patch(entity_id, EntityPatch::new(payload))
            .await
            .change_context(EntityServiceError)
    }
}
