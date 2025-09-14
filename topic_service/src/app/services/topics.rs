use error_stack::{Result, ResultExt};
use tracing::instrument;

use crate::{
    app::{
        models::Topic,
        pagination::Pagination,
        repository::{Repository, TopicFilter, TopicRepository},
    },
    error::TopicServiceError,
};

#[derive(Debug)]
pub struct TopicService<T> {
    repo: T,
}

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
    const PAGE_SIZE: usize = 25;

    pub fn new(repo: T) -> TopicService<T> {
        TopicService { repo }
    }

    #[instrument(skip_all)]
    pub async fn search(
        &self,
        name: Option<String>,
        description: Option<String>,
        pagination: Pagination,
    ) -> Result<Vec<Topic>, TopicServiceError> {
        let topic_repo = self.repo.topics();
        let page = pagination.page;
        let page_size = pagination.page_size.unwrap_or(Self::PAGE_SIZE);

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
}
