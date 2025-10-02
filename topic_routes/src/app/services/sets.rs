use crate::app::services::ResourceOutcome;
use crate::error::{AppResult, SetServiceError};
use engine::Engine;
use engine::models::{Set, SetId, TopicId};
use engine::repository::sets::ExistingSetRepository;
use engine::repository::topics::ExistingTopicRepository;
use engine::repository::{SetsRepository, TopicsRepository};
use engine::search_filters::SetSearchCriteria;
use error_stack::ResultExt;
use itertools::Itertools;
use serde_json::Value;
use tracing::{Span, debug, info, instrument};

#[derive(Debug, Clone)]
pub struct SetService<T> {
    engine: T,
}

impl<T> SetService<T>
where
    T: Engine,
{
    pub fn new(repo: T) -> Self {
        Self { engine: repo }
    }

    #[instrument(skip_all, name = "service#search", fields(sets_found))]
    pub async fn search(
        &self,
        topic_id: TopicId,
        search_criteria: SetSearchCriteria,
    ) -> AppResult<Option<Vec<Set>>, SetServiceError> {
        info!("searching sets..");

        let Some(topic) = self
            .engine
            .topics()
            .expect_existing(topic_id)
            .await
            .change_context(SetServiceError)?
        else {
            debug!("topic does not exist");
            return Ok(None);
        };

        let sets = topic
            .sets()
            .search(search_criteria)
            .await
            .change_context(SetServiceError)?;

        Span::current().record("sets_found", sets.len());

        info!("search complete");

        Ok(Some(sets))
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
        let Some(topic) = self
            .engine
            .topics()
            .expect_existing(topic_id)
            .await
            .change_context(SetServiceError)?
        else {
            debug!("topic does not exist");
            return Ok(None);
        };

        debug!("topic does exist, creating set");

        let new_set = topic
            .sets()
            .create(set_name, initial_entity_payloads.unwrap_or_default())
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
        let Some(topic) = self
            .engine
            .topics()
            .expect_existing(topic_id)
            .await
            .change_context(SetServiceError)?
        else {
            debug!("topic does not exist");
            return Ok(None);
        };

        topic
            .sets()
            .find(set_id)
            .await
            .change_context(SetServiceError)
    }

    #[instrument(skip_all, name = "service#delete")]
    pub async fn delete(
        &self,
        topic_id: TopicId,
        set_id: SetId,
    ) -> AppResult<ResourceOutcome, SetServiceError> {
        let Some(topic) = self
            .engine
            .topics()
            .expect_existing(topic_id)
            .await
            .change_context(SetServiceError)?
        else {
            debug!("topic does not exist");
            return Ok(ResourceOutcome::NotFound);
        };

        let Some(set) = topic
            .sets()
            .expect_existing(set_id)
            .await
            .change_context(SetServiceError)?
        else {
            debug!("set does not exist");
            return Ok(ResourceOutcome::NotFound);
        };

        info!("deleting set");
        set.delete().await.change_context(SetServiceError)?;
        Ok(ResourceOutcome::Found)
    }
}
