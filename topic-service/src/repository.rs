use crate::error::TopicRepoError;
use crate::model::Topic;
use engine::Pagination;
use engine::id::TopicId;
use error_stack::Report;
use mongodb::Client;
use optional_field::Field;

pub type RepoResult<T> = Result<T, Report<TopicRepoError>>;
pub type OptRepoResult<T> = Result<Option<T>, Report<TopicRepoError>>;

pub struct NewTopic {
    name: String,
    description: Option<String>,
}

impl NewTopic {
    pub fn new(name: String, description: Option<String>) -> NewTopic {
        Self { name, description }
    }
}

pub struct TopicPatch {
    name: Option<String>,
    description: Field<String>,
}

impl TopicPatch {
    pub fn new(name: Option<String>, description: Field<String>) -> TopicPatch {
        Self { name, description }
    }
}

#[derive(Debug, Clone)]
pub struct TopicRepo {
    client: Client,
}

impl TopicRepo {
    pub fn new(client: Client) -> TopicRepo {
        Self { client }
    }
}

impl TopicRepo {
    pub async fn get(&self, topic_id: TopicId) -> OptRepoResult<Topic> {
        todo!()
    }

    pub async fn list(&self, pagination: Pagination) -> RepoResult<Vec<Topic>> {
        todo!()
    }

    pub async fn create(&self, new_topic: NewTopic) -> RepoResult<Topic> {
        todo!()
    }

    pub async fn delete(&self, topic_id: TopicId) -> OptRepoResult<()> {
        todo!()
    }

    pub async fn patch(&self, topic_id: TopicId, patch: TopicPatch) -> OptRepoResult<Topic> {
        todo!()
    }
}
