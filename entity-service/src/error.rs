#[derive(Debug, thiserror::Error)]
#[error("entity service failed")]
pub struct EntityServiceError;

#[derive(Debug, thiserror::Error)]
pub enum EntityRepoError {
    #[error("failed to get topic")]
    Get,
    #[error("failed to create topic")]
    Create,
    #[error("failed to patch topic")]
    Patch,
    #[error("failed to delete topic")]
    Delete,
}
