use crate::error::{RepoResult, TopicRepoError};
use crate::models::Topic;
use crate::repository::{IdentifiersRepository, SetsRepository};
use optional_field::Field;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TopicUpdate {
    pub name: Option<String>,
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
