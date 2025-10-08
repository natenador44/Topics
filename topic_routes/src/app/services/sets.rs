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
    async fn get_set_repo(
        &self,
        topic_id: TopicId,
    ) -> AppResult<Option<impl SetsRepository>, SetServiceError> {
        let set_repo = self
            .engine
            .topics()
            .expect_existing(topic_id)
            .await
            .change_context(SetServiceError)?
            .map(|tr| tr.sets());

        Ok(set_repo)
    }

    async fn get_existing_set_repo(
        &self,
        topic_id: TopicId,
        set_id: SetId,
    ) -> AppResult<Option<impl ExistingSetRepository>, SetServiceError> {
        let Some(set_repo) = self.get_set_repo(topic_id).await? else {
            return Ok(None);
        };

        set_repo
            .expect_existing(set_id)
            .await
            .change_context(SetServiceError)
    }
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

        let Some(set_repo) = self.get_set_repo(topic_id).await? else {
            return Ok(None);
        };

        let sets = set_repo
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
        description: Option<String>,
        initial_entity_payloads: Option<Vec<Value>>,
    ) -> AppResult<Option<Set>, SetServiceError> {
        let Some(set_repo) = self.get_set_repo(topic_id).await? else {
            return Ok(None);
        };

        debug!("topic does exist, creating set");

        let new_set = set_repo
            .create(
                set_name,
                description,
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
        let Some(set_repo) = self.get_set_repo(topic_id).await? else {
            return Ok(None);
        };

        set_repo.find(set_id).await.change_context(SetServiceError)
    }

    #[instrument(skip_all, name = "service#delete")]
    pub async fn delete(
        &self,
        topic_id: TopicId,
        set_id: SetId,
    ) -> AppResult<ResourceOutcome, SetServiceError> {
        let Some(set_repo) = self.get_existing_set_repo(topic_id, set_id).await? else {
            return Ok(ResourceOutcome::NotFound);
        };

        info!("deleting set");
        set_repo.delete().await.change_context(SetServiceError)?;
        Ok(ResourceOutcome::Found)
    }
}
