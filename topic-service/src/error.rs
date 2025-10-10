#[derive(Debug, thiserror::Error)]
#[error("topic service failed")]
pub struct TopicServiceError;

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
