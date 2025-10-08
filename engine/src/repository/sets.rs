use crate::error::{RepoResult, SetRepoError};
use crate::models::Set;
use crate::repository::EntitiesRepository;
use optional_field::Field;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SetUpdate {
    pub name: Option<String>,
    pub description: Field<String>,
}

pub trait ExistingSetRepository {
    type EntitiesRepo: EntitiesRepository + Send + Sync + 'static;

    fn entities(&self) -> Self::EntitiesRepo;

    fn delete(&self) -> impl Future<Output = RepoResult<(), SetRepoError>> + Send;
    fn update(&self, set: SetUpdate) -> impl Future<Output = RepoResult<Set, SetRepoError>> + Send;
}
