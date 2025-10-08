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
    #[error("failed to check if set exists")]
    Exists,
    #[error("failed to update set")]
    Update,
}

#[derive(Debug, thiserror::Error)]
pub enum EntityRepoError {
    #[error("failed to create entity")]
    Create,
    #[error("failed to get entity")]
    Get,
    #[error("failed to delete entity")]
    Delete,
    #[error("failed to search entities")]
    Search,
    #[error("failed to check if entity exists")]
    Exists,
    #[error("failed to delete all entities in set")]
    DeleteAllInSet,
    #[error("failed to update entity")]
    Update,
}
