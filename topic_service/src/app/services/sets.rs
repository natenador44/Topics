use error_stack::{IntoReport, ResultExt};
use serde_json::Value;

use crate::{
    app::models::{Entity, TopicId, TopicSetId},
    error::{AppResult, SetServiceError},
};
use crate::app::models::{EntityId, TopicSet};
use crate::app::repository::{Repository, SetRepository, TopicRepository};

#[derive(Debug, Clone)]
pub struct SetService<T> {
    repo: T,
}

#[derive(Debug, thiserror::Error)]
enum TopicInteractionError {
    #[error("topic not found")]
    NotFound,
    #[error("topic repo returned an error")]
    RepoError,
}

impl<T> SetService<T> 
    where T: Repository,
{
    async fn topic_exists(&self, topic_id: TopicId) -> AppResult<bool, TopicInteractionError> {
        self.repo.topics().exists(topic_id).await
            .change_context(TopicInteractionError::RepoError)
    }
    
}

impl<T> SetService<T>
where
    T: Repository,
{
    pub fn new(repo: T) -> Self {
        Self { repo }
    }

    pub async fn create(
        &self,
        topic_id: TopicId,
        set_name: String,
        initial_entity_payloads: Vec<Value>,
    ) -> AppResult<TopicSet, SetServiceError> { // think about changing what `TopicSEt` contains.. maybe just entity ids instead of the full things
        
        let topic_exists = self.topic_exists(topic_id).await
            .change_context(SetServiceError)?;
        
        if !topic_exists {
            return Err(TopicInteractionError::NotFound.into_report())
                .change_context(SetServiceError);
        };
       
        
        let new_set = self.repo.sets().create(topic_id, set_name, initial_entity_payloads).await
            .change_context(SetServiceError)?;
        
        
        
        Ok(new_set)
    }
}
