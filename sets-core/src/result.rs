use error_stack::Report;

pub type RepoResult<T> = Result<T, Report<SetRepoError>>;
pub type OptRepoResult<T> = Result<Option<T>, Report<SetRepoError>>;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SetRepoError {
    #[error("failed to get set")]
    Get,
    #[error("failed to create set: {0}")]
    Create(Reason),
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Reason {
    #[error("topic associated with set was not found")]
    TopicNotFound,
    #[error("database call failed")]
    Db,
}
