use error_stack::Report;
use mongodb::Client;
use optional_field::Field;
use serde_json::Value;
use engine::id::{EntityId, SetId, TopicId};
use engine::Pagination;
use crate::error::EntityRepoError;
use crate::model::Entity;

pub type RepoResult<T> = Result<T, Report<EntityRepoError>>;
pub type OptRepoResult<T> = Result<Option<T>, Report<EntityRepoError>>;

pub struct NewEntity {
    topic_id: TopicId,
    set_id: SetId,
    payload: Value,
}

impl NewEntity {
    pub fn new(topic_id: TopicId, set_id: SetId, payload: Value) -> NewEntity {
        Self { topic_id, set_id, payload }
    }
}

pub struct EntityPatch {
    payload: Option<Value>,
}

impl EntityPatch {
    pub fn new(payload: Option<Value>) -> EntityPatch {
        Self { payload }
    }
}

#[derive(Debug, Clone)]
pub struct EntityRepo {
    client: Client,
}

impl EntityRepo {
    pub fn new(client: Client) -> EntityRepo {
        EntityRepo { client }
    }
}

impl EntityRepo {
    pub async fn get(&self, entity_id: EntityId) -> OptRepoResult<Entity> {
        todo!()
    }

    pub async fn list(&self, pagination: Pagination) -> RepoResult<Vec<Entity>> {
        todo!()
    }

    pub async fn create(&self, new_entity: NewEntity) -> RepoResult<Entity> {
        todo!()
    }

    pub async fn delete(&self, entity_id: EntityId) -> OptRepoResult<()> {
        todo!()
    }

    pub async fn patch(&self, entity_id: EntityId, patch: EntityPatch) -> OptRepoResult<Entity> {
        todo!()
    }
}
