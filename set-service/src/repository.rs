use error_stack::Report;
use mongodb::Client;
use optional_field::Field;
use engine::id::{SetId, TopicId};
use engine::Pagination;
use crate::error::SetRepoError;
use crate::model::Set;

pub type RepoResult<T> = Result<T, Report<SetRepoError>>;
pub type OptRepoResult<T> = Result<Option<T>, Report<SetRepoError>>;

pub struct NewSet {
    topic_id: TopicId,
    name: String,
    description: Option<String>,
}

impl NewSet {
    pub fn new(topic_id: TopicId, name: String, description: Option<String>) -> NewSet {
        Self { topic_id, name, description }
    }
}

pub struct SetPatch {
    name: Option<String>,
    description: Field<String>,
}

impl SetPatch {
    pub fn new(name: Option<String>, description: Field<String>) -> SetPatch {
        Self { name, description }
    }
}

#[derive(Debug, Clone)]
pub struct SetRepo {
    client: Client,
}

impl SetRepo {
    pub fn new(client: Client) -> SetRepo {
        SetRepo { client }
    }
}

impl SetRepo {
    pub async fn get(&self, topic_id: SetId) -> OptRepoResult<Set> {
        todo!()
    }

    pub async fn list(&self, pagination: Pagination) -> RepoResult<Vec<Set>> {
        todo!()
    }

    pub async fn create(&self, new_topic: NewSet) -> RepoResult<Set> {
        todo!()
    }

    pub async fn delete(&self, topic_id: SetId) -> OptRepoResult<()> {
        todo!()
    }

    pub async fn patch(&self, topic_id: SetId, patch: SetPatch) -> OptRepoResult<Set> {
        todo!()
    }
}
