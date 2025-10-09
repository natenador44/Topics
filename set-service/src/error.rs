#[derive(Debug, thiserror::Error)]
#[error("topic service failed")]
pub struct SetServiceError;

#[derive(Debug, thiserror::Error)]
pub enum SetRepoError {
    #[error("failed to get topic")]
    Get,
    #[error("failed to create topic")]
    Create,
    #[error("failed to patch topic")]
    Patch,
    #[error("failed to delete topic")]
    Delete,
}
