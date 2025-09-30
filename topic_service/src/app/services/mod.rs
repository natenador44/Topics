mod sets;
mod topics;

pub use sets::{SetSearch, SetService};
pub use topics::TopicService;

#[cfg(test)]
pub use topics::DEFAULT_TOPIC_SEARCH_PAGE_SIZE;

use crate::{app::repository::Repository, error::AppResult};
use error_stack::ResultExt;

use crate::app::repository::file::FileRepo;

#[derive(Debug, Clone)]
pub struct Service<T> {
    pub topics: TopicService<T>,
    pub sets: SetService<T>,
}

/// Used for service methods that only need to report if
/// a resource exists (or existed), but don't need to get a representation of that data.
/// This can be used by the route layer to return the correct response to the user
#[derive(Debug)]
pub enum ResourceOutcome {
    Found,
    NotFound,
}

#[derive(Debug, thiserror::Error)]
#[error("failed to create service")]
pub struct ServiceBuildError;

pub fn build() -> AppResult<Service<FileRepo>, ServiceBuildError> {
    let repo = init_repo()?;
    Ok(Service {
        topics: TopicService::new(repo.clone()),
        sets: SetService::new(repo.clone()),
    })
}

fn init_repo() -> AppResult<FileRepo, ServiceBuildError> {
    FileRepo::init(tokio::runtime::Handle::current()).change_context(ServiceBuildError)
}
