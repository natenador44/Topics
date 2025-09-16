mod topics;

pub use topics::TopicService;

#[cfg(test)]
pub use topics::DEFAULT_TOPIC_SEARCH_PAGE_SIZE;

use error_stack::ResultExt;
use crate::error::AppResult;

use crate::app::repository::file::FileRepo;

#[derive(Debug)]
pub struct Service<T> {
    pub topics: TopicService<T>,
}

impl<T> Clone for Service<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            topics: self.topics.clone(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("failed to create service")]
pub struct ServiceBuildError;

pub fn build() -> AppResult<Service<FileRepo>, ServiceBuildError> {
    Ok(Service {
        topics: TopicService::new(
            FileRepo::init(tokio::runtime::Handle::current()).change_context(ServiceBuildError)?,
        ),
    })
}
