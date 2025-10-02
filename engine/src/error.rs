use error_stack::Report;
use std::error::Error;

pub type RepoResult<T, E> = Result<T, Report<E>>;
#[derive(Debug, thiserror::Error)]
pub enum TopicRepoError {
    #[error("failed to search topics")]
    Search,
    #[error("error occurred while finding topic")]
    Get,
    #[error("failed to create new topic")]
    Create,
    #[error("failed to delete topic")]
    Delete,
    #[error("failed to update topic")]
    Update,
    #[error("failed to check if topic exists")]
    Exists,
}

#[derive(Debug, thiserror::Error)]
pub enum SetRepoError {
    #[error("failed to create set")]
    Create,
    #[error("failed to get set")]
    Get,
    #[error("failed to delete set")]
    Delete,
    #[error("failed to search sets")]
    Search,
}
