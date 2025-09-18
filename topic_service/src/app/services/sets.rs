use crate::app::models::{EntityId, TopicSet};
use crate::app::repository::{Repository, SetRepository, TopicRepoError, TopicRepository};
use crate::{
    app::models::{Entity, TopicId, TopicSetId},
    error::{AppResult, SetServiceError},
};
use error_stack::{IntoReport, ResultExt};
use serde_json::Value;
use tracing::{debug, info, instrument};

#[derive(Debug, Clone)]
pub struct SetService<T> {
    repo: T,
}

impl<T> SetService<T>
where
    T: Repository,
{
    async fn topic_exists(&self, topic_id: TopicId) -> AppResult<bool, TopicRepoError> {
        self.repo.topics().exists(topic_id).await
    }
}

impl<T> SetService<T>
where
    T: Repository,
{
    pub fn new(repo: T) -> Self {
        Self { repo }
    }

    #[instrument(skip_all, name = "service#create")]
    pub async fn create(
        &self,
        topic_id: TopicId,
        set_name: String,
        initial_entity_payloads: Option<Vec<Value>>,
    ) -> AppResult<TopicSet, SetServiceError> {
        info!("creating set...");
        debug!("checking if topic exists");
        let topic_exists = self
            .topic_exists(topic_id)
            .await
            .change_context(SetServiceError::TopicServiceError)?;

        if !topic_exists {
            debug!("topic does not exist");
            return Err(SetServiceError::TopicNotFound.into_report());
        };

        debug!("topic does exist, creating set");

        let new_set = self
            .repo
            .sets()
            .create(
                topic_id,
                set_name,
                initial_entity_payloads.unwrap_or_default(),
            )
            .await
            .change_context(SetServiceError::Create)?;

        info!("set created successfully");
        Ok(new_set)
    }
}
