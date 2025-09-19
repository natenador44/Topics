use crate::app::models::{EntityId, Set};
use crate::app::repository::{Repository, SetRepository, TopicRepoError, TopicRepository};
use crate::{
    app::models::{Entity, SetId, TopicId},
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
    pub fn new(repo: T) -> Self {
        Self { repo }
    }

    /// Create a new `Set` under the given `Topic` (`topic_id`).
    /// `initial_entity_payloads` will be the initial contents of the set.
    /// Returns `Ok(Some(Set))` if the `topic_id` exists and no errors occur.
    /// Returns `Ok(None)` if the `topic_id` does not exist and no errors occur.
    #[instrument(skip_all, name = "service#create")]
    pub async fn create(
        &self,
        topic_id: TopicId,
        set_name: String,
        initial_entity_payloads: Option<Vec<Value>>,
    ) -> AppResult<Option<Set>, SetServiceError> {
        info!("creating set...");
        debug!("checking if topic exists");
        let topic_exists = self
            .topic_exists(topic_id)
            .await
            .change_context(SetServiceError)?;

        if !topic_exists {
            debug!("topic does not exist");
            return Ok(None);
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
            .change_context(SetServiceError)?;

        info!("set created successfully");
        Ok(Some(new_set))
    }

    #[instrument(skip_all, name = "service#get")]
    pub async fn get(
        &self,
        topic_id: TopicId,
        set_id: SetId,
    ) -> AppResult<Option<Set>, SetServiceError> {
        let topic_exists = self
            .topic_exists(topic_id)
            .await
            .change_context(SetServiceError)?;

        if !topic_exists {
            return Ok(None);
        }

        self.repo
            .sets()
            .get(topic_id, set_id)
            .await
            .change_context(SetServiceError)
    }

    /*
    Checking this way adds more database reads (potentially, we'll see when the cache comes in)
    It also doesn't rely on 'set' data to confirm the existence of a topic, but instead goes straight
    to the source. This can be a good thing and a bad thing, but only a bad thing if we aren't keeping
    the 'set' data and the 'topic' data in sync. How this is done may change in the future.
     */
    // TODO see if this could be an extractor instead
    async fn topic_exists(&self, topic_id: TopicId) -> AppResult<bool, TopicRepoError> {
        self.repo.topics().exists(topic_id).await
    }
}
