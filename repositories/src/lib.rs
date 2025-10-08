use error_stack::Report;

#[cfg(feature = "postgres")]
pub mod postgres;

pub type RepoInitResult<T> = Result<T, Report<RepoInitErr>>;

#[derive(Debug, thiserror::Error)]
#[error("failed to initialize repository")]
pub struct RepoInitErr;
