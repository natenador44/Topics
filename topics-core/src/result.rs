use error_stack::Report;

pub type RepoResult<T> = Result<T, Report<TopicRepoError>>;
pub type OptRepoResult<T> = Result<Option<T>, Report<TopicRepoError>>;

#[derive(Debug, thiserror::Error)]
pub enum TopicRepoError {
    #[error("failed to get topic")]
    Get,
    #[error("failed to list topics")]
    List,
    #[error("failed to create topic")]
    Create,
    #[error("failed to patch topic")]
    Patch,
    #[error("failed to delete topic")]
    Delete,
}
