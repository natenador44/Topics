use error_stack::Report;

pub type RepoResult<T> = Result<T, Report<SetRepoError>>;
pub type OptRepoResult<T> = Result<Option<T>, Report<SetRepoError>>;

#[derive(Debug, thiserror::Error)]
pub enum SetRepoError {
    #[error("failed to get set")]
    Get,
}
