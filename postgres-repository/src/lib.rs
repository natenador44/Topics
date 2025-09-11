use std::sync::Arc;

use repository::{IdentifierRepository, Repository, SetRepository, TopicRepository};

#[derive(Clone, Debug)]
pub struct PostgresRepo {
    inner: Arc<RepoInner>,
}

impl PostgresRepo {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RepoInner {
                topic_repo: PostgresTopicRepo {},
                identifier_repo: PostgresIdentifierRepo {},
                set_repo: PostgresSetRepo {},
            }),
        }
    }
}

#[derive(Debug)]
struct RepoInner {
    topic_repo: PostgresTopicRepo,
    identifier_repo: PostgresIdentifierRepo,
    set_repo: PostgresSetRepo,
}

impl Repository for PostgresRepo {
    type TopicRepo = PostgresTopicRepo;
    type IdentifierRepo = PostgresIdentifierRepo;
    type SetRepo = PostgresSetRepo;

    fn topic_repository(&self) -> &Self::TopicRepo {
        &self.inner.topic_repo
    }

    fn identifier_repository(&self) -> &Self::IdentifierRepo {
        &self.inner.identifier_repo
    }

    fn set_repository(&self) -> &Self::SetRepo {
        &self.inner.set_repo
    }
}

#[derive(Debug)]
pub struct PostgresTopicRepo {}

impl TopicRepository for PostgresTopicRepo {}

#[derive(Debug)]
pub struct PostgresIdentifierRepo {}

impl IdentifierRepository for PostgresIdentifierRepo {}

#[derive(Debug)]
pub struct PostgresSetRepo {}
impl SetRepository for PostgresSetRepo {}
