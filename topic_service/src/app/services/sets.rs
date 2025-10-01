use crate::app::models::{IdentifierId, Set};
use crate::app::pagination::Pagination;
use crate::app::repository::{Repository, SetRepository, TopicRepoError, TopicRepository};
use crate::app::search_filter::{SearchCriteria, SearchFilter, Tag};
use crate::app::services::ResourceOutcome;
use crate::{
    app::models::{SetId, TopicId},
    error::{AppResult, SetServiceError},
};
use error_stack::ResultExt;
use serde_json::Value;
use tracing::{Span, debug, info, instrument};

#[derive(Debug, Clone)]
pub struct SetService<T> {
    repo: T,
}

const DEFAULT_SET_PAGE_SIZE: usize = 10;

const MAX_SEARCH_FILTER_COUNT: usize = 3;
pub type SetSearchCriteria = SearchCriteria<SetSearch, MAX_SEARCH_FILTER_COUNT>;

pub enum SetSearch {
    Name(String),
    EntityText(String),
    Identifiers(Vec<IdentifierId>),
}

impl SearchFilter for SetSearch {
    const MAX_FILTER_COUNT: usize = MAX_SEARCH_FILTER_COUNT;
    type Criteria = SetSearchCriteria;

    fn tag(&self) -> Tag {
        match self {
            SetSearch::Name(_) => Tag::One,
            SetSearch::EntityText(_) => Tag::Two,
            SetSearch::Identifiers(_) => Tag::Four,
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

    #[instrument(skip_all, name = "service#search", fields(sets_found))]
    pub async fn search(
        &self,
        topic_id: TopicId,
        search_criteria: SetSearchCriteria,
    ) -> AppResult<Option<Vec<Set>>, SetServiceError> {
        info!("searching sets..");
        // check if topic exists here, or have the repo tell the server?
        // think about it..
        // if we're using postgres, what would the data look like?
        // we'd have some sql like "where topic_id = :topic_id [and filter [ and filter]]", or
        // it would be a join of some kind. That would just result in an empty list, which is
        // not a 404 in this case. So I think I need to check first.
        let topic_exists = self
            .topic_exists(topic_id)
            .await
            .change_context(SetServiceError)?;

        if !topic_exists {
            debug!("topic does not exist");
            return Ok(None);
        }

        let sets = self
            .repo
            .sets()
            .search(topic_id, search_criteria)
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
        info!("creating set...");
        let topic_exists = self
            .topic_exists(topic_id)
            .await
            .change_context(SetServiceError)?;

        if !topic_exists {
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

        let set = self
            .repo
            .sets()
            .get(topic_id, set_id)
            .await
            .change_context(SetServiceError)?;
        Ok(Some(set))
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
    #[instrument(skip(self), ret)]
    async fn topic_exists(&self, topic_id: TopicId) -> AppResult<bool, TopicRepoError> {
        debug!("checking if topic exists..");
        self.repo.topics().exists(topic_id).await
    }
}
