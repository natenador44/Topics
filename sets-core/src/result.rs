use error_stack::Report;

pub type RepoResult<T> = Result<T, Report<SetRepoError>>;
pub type OptRepoResult<T> = Result<Option<T>, Report<SetRepoError>>;

#[derive(Debug, thiserror::Error, PartialEq, Eq, Copy, Clone)]
pub enum SetRepoError {
    #[error("failed to get set: {0}")]
    Get(Reason),
    #[error("failed to create set: {0}")]
    Create(Reason),
    #[error("failed to get list of sets: {0}")]
    List(Reason),
    #[error("failed to create many sets: {0}")]
    CreateMany(Reason),
    #[error("failed to patch set: {0}")]
    Patch(Reason),
    #[error("failed to delete set: {0}")]
    Delete(Reason),
}

#[derive(Debug, thiserror::Error, PartialEq, Eq, Copy, Clone)]
pub enum Reason {
    #[error("topic associated with set was not found")]
    TopicNotFound,
    #[error("database call failed")]
    Db,
    #[error("input failed validation")]
    Validation,
}
