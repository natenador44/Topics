use crate::app::models::TopicId;
use crate::{
    app::{
        models::Topic,
        pagination::Pagination,
        repository::{Repository, TopicFilter, TopicRepository},
    },
    error::TopicServiceError,
};
use error_stack::{Result, ResultExt};
use tracing::instrument;
use uuid::Uuid;

pub const DEFAULT_TOPIC_SEARCH_PAGE_SIZE: usize = 25;

#[derive(Debug)]
pub struct TopicService<T> {
    repo: T,
}

impl<T> TopicService<T> {}

impl<T> Clone for TopicService<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            repo: self.repo.clone(),
        }
    }
}

impl<T: Repository> TopicService<T> {
    pub fn new(repo: T) -> TopicService<T> {
        TopicService { repo }
    }

    #[instrument(skip_all, ret(level = "debug"), name = "service#search")]
    pub async fn search(
        &self,
        name: Option<String>,
        description: Option<String>,
        pagination: Pagination,
    ) -> Result<Vec<Topic>, TopicServiceError> {
        let topic_repo = self.repo.topics();
        let page = pagination.page;
        let page_size = pagination
            .page_size
            .unwrap_or(DEFAULT_TOPIC_SEARCH_PAGE_SIZE);

        let filters = match (name, description) {
            (Some(n), None) => vec![TopicFilter::Name(n)],
            (None, Some(d)) => vec![TopicFilter::Description(d)],
            (Some(n), Some(d)) => vec![TopicFilter::Name(n), TopicFilter::Description(d)],
            (None, None) => vec![],
        };

        let topics = topic_repo
            .search(page, page_size, filters)
            .await
            .change_context(TopicServiceError)?;

        Ok(topics)
    }

    #[instrument(skip_all, ret(level = "debug"), name = "service#get_by_id")]
    pub async fn get(&self, topic_id: TopicId) -> Result<Option<Topic>, TopicServiceError> {
        let topic = self
            .repo
            .topics()
            .get(topic_id)
            .await
            .change_context(TopicServiceError)?;
        Ok(topic)
    }

    #[instrument(skip_all, ret(level = "debug"), name = "service#create")]
    pub async fn create(
        &self,
        name: String,
        description: Option<String>,
    ) -> Result<TopicId, TopicServiceError> {
        let new_id = self
            .repo
            .topics()
            .create(name, description)
            .await
            .change_context(TopicServiceError)?;

        Ok(new_id)
    }

    #[instrument(skip_all, ret(level = "debug"), name = "service#delete")]
    pub async fn delete(&self, topic_id: TopicId) -> Result<(), TopicServiceError> {
        self.repo
            .topics()
            .delete(topic_id)
            .await
            .change_context(TopicServiceError)
    }
}
