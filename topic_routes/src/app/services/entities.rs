use crate::app::services::ResourceOutcome;
use crate::error::{AppResult, EntityServiceError};
use axum::response::Response;
use engine::Engine;
use engine::models::{Entity, EntityId, SetId, TopicId};
use engine::repository::entities::ExistingEntityRepository;
use engine::repository::sets::ExistingSetRepository;
use engine::repository::topics::ExistingTopicRepository;
use engine::repository::{EntitiesRepository, SetsRepository, TopicsRepository};
use engine::search_filters::EntitySearchCriteria;
use error_stack::ResultExt;
use serde_json::Value;
use tracing::{info, instrument};

#[derive(Debug, Clone)]
pub struct EntityService<T> {
    engine: T,
}

impl<T> EntityService<T>
where
    T: Engine,
{
    pub fn new(engine: T) -> Self {
        EntityService { engine }
    }

    pub async fn search(
        &self,
        topic_id: TopicId,
        set_id: SetId,
        search_criteria: EntitySearchCriteria,
    ) -> AppResult<Option<Vec<Entity>>, EntityServiceError> {
        let Some(entity_repo) = self.get_entity_repo(topic_id, set_id).await? else {
            return Ok(None);
        };

        let entities = entity_repo
            .search(search_criteria)
            .await
            .change_context(EntityServiceError)?;

        Ok(Some(entities))
    }

    pub async fn get(
        &self,
        topic_id: TopicId,
        set_id: SetId,
        entity_id: EntityId,
    ) -> AppResult<Option<Entity>, EntityServiceError> {
        let Some(entity_repo) = self.get_entity_repo(topic_id, set_id).await? else {
            return Ok(None);
        };

        Ok(entity_repo
            .find(entity_id)
            .await
            .change_context(EntityServiceError)?)
    }

    pub async fn create_in_set(
        &self,
        topic_id: TopicId,
        set_id: SetId,
        payload: Value,
    ) -> AppResult<Option<Entity>, EntityServiceError> {
        let Some(entity_repo) = self.get_entity_repo(topic_id, set_id).await? else {
            return Ok(None);
        };

        entity_repo
            .create(payload)
            .await
            .change_context(EntityServiceError)
            .map(Some)
    }

    pub async fn remove_from_set(
        &self,
        topic_id: TopicId,
        set_id: SetId,
        entity_id: EntityId,
    ) -> AppResult<ResourceOutcome, EntityServiceError> {
        let Some(existing_entity_repo) = self
            .get_existing_entity_repo(topic_id, set_id, entity_id)
            .await?
        else {
            return Ok(ResourceOutcome::NotFound);
        };

        existing_entity_repo
            .delete()
            .await
            .change_context(EntityServiceError)
            .map(|_| ResourceOutcome::Found)
    }
}
impl<T> EntityService<T>
where
    T: Engine,
{
    #[instrument(skip_all)]
    async fn get_entity_repo(
        &self,
        topic_id: TopicId,
        set_id: SetId,
    ) -> AppResult<Option<impl EntitiesRepository>, EntityServiceError> {
        let Some(topic) = self
            .engine
            .topics()
            .expect_existing(topic_id)
            .await
            .change_context(EntityServiceError)?
        else {
            info!("topic id not found");
            return Ok(None);
        };

        let Some(set) = topic
            .sets()
            .expect_existing(set_id)
            .await
            .change_context(EntityServiceError)?
        else {
            info!("set id not found");
            return Ok(None);
        };

        Ok(Some(set.entities()))
    }

    #[instrument(skip_all)]
    async fn get_existing_entity_repo(
        &self,
        topic_id: TopicId,
        set_id: SetId,
        entity_id: EntityId,
    ) -> AppResult<Option<impl ExistingEntityRepository>, EntityServiceError> {
        let Some(entity_repo) = self.get_entity_repo(topic_id, set_id).await? else {
            return Ok(None);
        };

        entity_repo
            .expect_existing(entity_id)
            .await
            .change_context(EntityServiceError)
    }
}
