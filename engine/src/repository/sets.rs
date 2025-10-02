use crate::error::{RepoResult, SetRepoError};
use crate::models::Set;
use crate::repository::EntitiesRepository;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SetUpdate {}

pub trait ExistingSetRepository {
    type EntitiesRepo: EntitiesRepository + Send + Sync + 'static;

    fn entities(&self) -> Self::EntitiesRepo;

    fn delete(&self) -> impl Future<Output = RepoResult<(), SetRepoError>> + Send;
    fn update(&self, set: SetUpdate) -> impl Future<Output = RepoResult<Set, SetRepoError>> + Send;
}
