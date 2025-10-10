use crate::error::TopicRepoError;
use crate::filter::TopicListCriteria;
use crate::model::Topic;
use chrono::{DateTime, Utc};
use engine::id::TopicId;
use error_stack::{Report, ResultExt};
use mongodb::bson::{Bson, Document, doc};
use mongodb::options::{FindOneAndUpdateOptions, FindOptions, ReturnDocument, UpdateModifications};
use mongodb::{Client, Database};
use optional_field::Field;
use serde::Serialize;
use tokio_stream::StreamExt;
use tracing::debug;

pub type RepoResult<T> = Result<T, Report<TopicRepoError>>;
pub type OptRepoResult<T> = Result<Option<T>, Report<TopicRepoError>>;

pub struct NewTopicRequest {
    name: String,
    description: Option<String>,
}

impl NewTopicRequest {
    pub fn new(name: String, description: Option<String>) -> NewTopicRequest {
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
    db: Database,
}

impl TopicRepo {
    pub fn new(client: Client) -> TopicRepo {
        Self {
            db: client.database("topics"),
        }
    }
}

#[derive(Debug, Serialize)]
struct NewTopic {
    name: String,
    description: Option<String>,
    created: DateTime<Utc>,
}

impl TopicRepo {
    pub async fn get(&self, topic_id: TopicId) -> OptRepoResult<Topic> {
        self.db
            .collection::<Topic>("topics")
            .find_one(doc! { "_id": topic_id })
            .await
            .change_context(TopicRepoError::Get)
    }

    pub async fn list(&self, search_criteria: TopicListCriteria) -> RepoResult<Vec<Topic>> {
        let actual_page = search_criteria
            .page()
            .saturating_sub(1)
            .checked_mul(search_criteria.page_size())
            .ok_or(TopicRepoError::List)
            .attach_with(|| {
                format!(
                    "invalid page ({}) and page size ({})",
                    search_criteria.page(),
                    search_criteria.page_size()
                )
            })?;

        let options = FindOptions::builder()
            .skip(actual_page)
            .limit(search_criteria.page_size().try_into().ok())
            .build();

        self.db
            .collection::<Topic>("topics")
            .find(Document::default())
            .with_options(options)
            .await
            .change_context(TopicRepoError::List)?
            .collect::<Result<_, _>>()
            .await
            .change_context(TopicRepoError::List)
    }

    pub async fn create(&self, new_topic: NewTopicRequest) -> RepoResult<Topic> {
        let topic = NewTopic {
            name: new_topic.name,
            description: new_topic.description,
            created: Utc::now(),
        };

        let result = self
            .db
            .collection::<NewTopic>("topics")
            .insert_one(&topic)
            .await
            .change_context(TopicRepoError::Create)?;

        Ok(Topic {
            id: TopicId::new(
                result
                    .inserted_id
                    .as_object_id()
                    .ok_or(TopicRepoError::Create)?,
            ),
            name: topic.name,
            description: topic.description,
            created: topic.created,
            updated: None,
        })
    }

    pub async fn delete(&self, topic_id: TopicId) -> OptRepoResult<()> {
        let result = self
            .db
            .collection::<Topic>("topics")
            .delete_one(doc! { "_id": topic_id })
            .await
            .change_context(TopicRepoError::Delete)?;

        Ok((result.deleted_count > 0).then_some(()))
    }

    pub async fn patch(&self, topic_id: TopicId, patch: TopicPatch) -> OptRepoResult<Topic> {
        let mut update_document = Document::new();
        if let Some(name) = patch.name {
            update_document.insert("name", name);
        }

        if let Field::Present(desc) = patch.description {
            match desc {
                Some(d) => {
                    update_document.insert("description", d);
                }
                None => {
                    update_document.insert("description", Bson::Null);
                }
            }
        }

        update_document.insert("updated", Utc::now().to_rfc3339());

        debug!("Updating document {:?}", update_document);

        let options = FindOneAndUpdateOptions::builder()
            .return_document(ReturnDocument::After)
            .build();

        self.db
            .collection::<Topic>("topics")
            .find_one_and_update(doc! { "_id": topic_id }, doc! { "$set": update_document })
            .with_options(options)
            .await
            .change_context(TopicRepoError::Patch)
    }
}
