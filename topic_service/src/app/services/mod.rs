mod topics;

#[cfg(feature = "postgres-repo-full")]
use postgres_repository::PostgresRepo;

use crate::app::services::topics::TopicService;

#[derive(Debug)]
pub struct Service<T> {
    pub topic_service: TopicService<T>,
}

impl<T> Clone for Service<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            topic_service: self.topic_service.clone(),
        }
    }
}

#[cfg(feature = "postgres-repo-full")]
pub fn build() -> Service<PostgresRepo> {
    Service {
        topic_service: TopicService::new(PostgresRepo::new()),
    }
}
