use crate::error::{RepoResult, TopicRepoError};
use crate::models::Topic;
use crate::repository::{IdentifiersRepository, SetsRepository};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TopicUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
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
