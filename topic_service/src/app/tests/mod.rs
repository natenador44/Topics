//! All tests in this module are intended to test the contract made by the API,
//! e.g. return codes, handling query parameters, handling path parameters.
use error_stack::Result;
use std::{ops::Deref, sync::Arc};

use crate::app::{
    models::Topic,
    repository::{
        IdentifierRepository, MockIdentifierRepository, MockSetRepository, MockTopicRepository,
        Repository, SetRepository, TopicFilter, TopicRepoError, TopicRepository,
    },
};
use crate::app::models::TopicId;

mod v1;

#[derive(Debug, Clone)]
struct MockRepo {
    topic_repo: MockTopicRepoWrapper,
}

impl MockRepo {
    fn for_topics_test(topic_repo: MockTopicRepository) -> Self {
        Self {
            topic_repo: MockTopicRepoWrapper(Arc::new(topic_repo)),
        }
    }
}

impl Repository for MockRepo {
    type TopicRepo = MockTopicRepoWrapper;

    type IdentifierRepo = MockIdentifierRepoWrapper;

    type SetRepo = MockSetRepoWrapper;

    fn topics(&self) -> Self::TopicRepo {
        self.topic_repo.clone()
    }

    fn identifiers(&self) -> Self::IdentifierRepo {
        todo!()
    }

    fn sets(&self) -> Self::SetRepo {
        todo!()
    }
}

#[derive(Debug, Clone)]
struct MockTopicRepoWrapper(Arc<MockTopicRepository>);
impl Deref for MockTopicRepoWrapper {
    type Target = MockTopicRepository;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl TopicRepository for MockTopicRepoWrapper {
    async fn search(
        &self,
        page: usize,
        page_size: usize,
        filters: Vec<TopicFilter>,
    ) -> Result<Vec<Topic>, TopicRepoError> {
        self.0.search(page, page_size, filters).await
    }

    async fn get(&self, topic_id: TopicId) -> Result<Option<Topic>, TopicRepoError> {
        self.0.get(topic_id).await
    }
}

#[derive(Debug, Clone)]
struct MockIdentifierRepoWrapper(Arc<MockIdentifierRepository>);
impl Deref for MockIdentifierRepoWrapper {
    type Target = MockIdentifierRepository;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl IdentifierRepository for MockIdentifierRepoWrapper {}

#[derive(Debug, Clone)]
struct MockSetRepoWrapper(Arc<MockSetRepository>);
impl Deref for MockSetRepoWrapper {
    type Target = MockSetRepository;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl SetRepository for MockSetRepoWrapper {}
