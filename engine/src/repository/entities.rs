use crate::error::{EntityRepoError, RepoResult};
use crate::models::Entity;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EntityUpdate {
    pub payload: Option<Value>,
}

pub trait ExistingEntityRepository {
    fn delete(&self) -> impl Future<Output = RepoResult<(), EntityRepoError>> + Send;

    fn update(
        &self,
        entity_update: EntityUpdate,
    ) -> impl Future<Output = RepoResult<Entity, EntityRepoError>> + Send;
}
