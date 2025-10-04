use crate::patch_field_schema;
use crate::error::{RepoResult, TopicRepoError};
use crate::models::Topic;
use crate::repository::{IdentifiersRepository, SetsRepository};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use optional_field::Field;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct TopicUpdate {
    #[schema(schema_with = patch_field_schema)]
    pub name: Field<String>,
    #[schema(schema_with = patch_field_schema)]
    pub description: Field<String>,
}

pub trait ExistingTopicRepository {
    type SetRepo: SetsRepository + Send + Sync + 'static;
    type IdentifierRepo: IdentifiersRepository + Send + Sync + 'static;

    fn sets(&self) -> Self::SetRepo;
    fn identifiers(&self) -> Self::IdentifierRepo;

    fn delete(&self) -> impl Future<Output = RepoResult<(), TopicRepoError>> + Send;

    fn update(
        &self,
        topic: TopicUpdate,
    ) -> impl Future<Output = RepoResult<Topic, TopicRepoError>> + Send;
}
