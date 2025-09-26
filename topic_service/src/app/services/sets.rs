use crate::app::models::{IdentifierId, Set};
use crate::app::pagination::Pagination;
use crate::app::repository::{Repository, SetRepository, TopicRepoError, TopicRepository};
use crate::app::services::ResourceOutcome;
use crate::{
    app::models::{SetId, TopicId},
    error::{AppResult, SetServiceError},
};
use error_stack::ResultExt;
use serde_json::Value;
use tracing::{debug, info, instrument};
use crate::app::search_filter::{SearchCriteria, SearchFilter, Tag};

#[derive(Debug, Clone)]
pub struct SetService<T> {
    repo: T,
}

const DEFAULT_SET_PAGE_SIZE: usize = 10;

pub type SetSearchCriteria = SearchCriteria<SetSearchFilter, 3>;

pub enum SetSearchFilter {
    Name(String),
    EntityText(String),
    Identifiers(Vec<IdentifierId>),
}

impl SearchFilter for SetSearchFilter {
    type Criteria = SetSearchCriteria;

    fn tag(&self) -> Tag {
        match self {
            SetSearchFilter::Name(_) => Tag::One,
            SetSearchFilter::EntityText(_) => Tag::Two,
            SetSearchFilter::Identifiers(_) => Tag::Four,
        }
    }

    fn criteria(pagination: Pagination) -> Self::Criteria {
        SearchCriteria::new(pagination, DEFAULT_SET_PAGE_SIZE)
    }
}


// TODO have all `delete` operations return a not-found-like response if any of the
// requested resources do not exist

impl<T> SetService<T>
where
    T: Repository,
{
    pub fn new(repo: T) -> Self {
        Self { repo }
    }

    #[instrument(skip_all, name = "service#search")]
    pub async fn search(
        &self,
        topic_id: TopicId,
        search_criteria: SetSearchCriteria,
    ) -> AppResult<Option<Vec<Set>>, SetServiceError> {
        todo!()
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

    #[instrument(skip_all, name = "service#delete")]
    pub async fn delete(
        &self,
        topic_id: TopicId,
        set_id: SetId,
    ) -> AppResult<ResourceOutcome, SetServiceError> {
        let topic_exists = self
            .topic_exists(topic_id)
            .await
            .change_context(SetServiceError)?;

        if !topic_exists {
            return Ok(ResourceOutcome::NotFound);
        }
        self.repo
            .sets()
            .delete(topic_id, set_id)
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
