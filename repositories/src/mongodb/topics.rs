use bson::oid::ObjectId;
use bson::{Bson, Document, doc};
use chrono::{DateTime, Utc};
use error_stack::{Report, ResultExt};
use mongodb::options::{FindOneAndUpdateOptions, FindOptions, ReturnDocument};
use mongodb::{Client, Database};
use optional_field::Field;
use serde::{Deserialize, Serialize, Serializer};
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use tokio_stream::StreamExt;
use topics_core::TopicRepository;
use topics_core::list_filter::TopicListCriteria;
use topics_core::model::{NewTopic, PatchTopic, Topic};
use topics_core::result::{OptRepoResult, RepoResult, TopicRepoError};
use tracing::debug;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Eq, Clone)]
#[repr(transparent)]
#[schema(value_type = String)]
pub struct TopicId(#[serde(serialize_with = "obj_id_serialize")] ObjectId);

fn obj_id_serialize<S>(id: &ObjectId, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    id.to_hex().serialize(ser)
}

impl TopicId {
    pub fn new(id: ObjectId) -> Self {
        Self(id)
    }
}

impl Deref for TopicId {
    type Target = ObjectId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<TopicId> for Bson {
    fn from(value: TopicId) -> Self {
        value.0.into()
    }
}

impl Display for TopicId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Serialize)]
struct NewTopicCreated {
    name: String,
    description: Option<String>,
    created: DateTime<Utc>,
}

impl NewTopicCreated {
    fn new(name: String, description: Option<String>) -> Self {
        Self {
            name,
            description,
            created: Utc::now(),
        }
    }
}

pub enum ConnectionDetails {
    Url(String),
}

#[derive(Debug, thiserror::Error)]
#[error("failed to create client connection to mongodb instance")]
pub struct ConnectError;

#[derive(Debug, Serialize, Deserialize)]
struct MongoTopic {
    #[serde(rename = "_id")]
    id: TopicId,
    name: String,
    description: Option<String>,
    created: DateTime<Utc>,
    updated: Option<DateTime<Utc>>,
}

impl From<Topic<TopicId>> for MongoTopic {
    fn from(value: Topic<TopicId>) -> Self {
        Self {
            id: value.id,
            name: value.name,
            description: value.description,
            created: value.created,
            updated: value.updated,
        }
    }
}

impl From<MongoTopic> for Topic<TopicId> {
    fn from(value: MongoTopic) -> Self {
        Self {
            id: value.id,
            name: value.name,
            description: value.description,
            created: value.created,
            updated: value.updated,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TopicRepo {
    db: Database,
}

impl TopicRepo {
    pub async fn init(
        connection_details: ConnectionDetails,
    ) -> Result<TopicRepo, Report<ConnectError>> {
        let client = match connection_details {
            ConnectionDetails::Url(url) => Client::with_uri_str(url)
                .await
                .change_context(ConnectError)?,
        };

        Ok(Self {
            db: client.database("topics"),
        })
    }
}

impl TopicRepository for TopicRepo {
    type TopicId = TopicId;

    async fn get(&self, id: Self::TopicId) -> OptRepoResult<Topic<Self::TopicId>> {
        self.db
            .collection::<MongoTopic>("topics")
            .find_one(doc! { "_id": id })
            .await
            .change_context(TopicRepoError::Get)
            .map(|t| t.map(From::from))
    }

    async fn list(
        &self,
        list_criteria: TopicListCriteria,
    ) -> RepoResult<Vec<Topic<Self::TopicId>>> {
        let actual_page = list_criteria
            .page()
            .saturating_sub(1)
            .checked_mul(list_criteria.page_size())
            .ok_or(TopicRepoError::List)
            .attach_with(|| {
                format!(
                    "invalid page ({}) and page size ({})",
                    list_criteria.page(),
                    list_criteria.page_size()
                )
            })?;

        let options = FindOptions::builder()
            .skip(actual_page)
            .limit(list_criteria.page_size().try_into().ok())
            .build();

        self.db
            .collection::<MongoTopic>("topics")
            .find(Document::default())
            .with_options(options)
            .await
            .change_context(TopicRepoError::List)?
            .map(|t| t.map(From::from))
            .collect::<Result<_, _>>()
            .await
            .change_context(TopicRepoError::List)
    }

    async fn create(&self, new_topic: NewTopic) -> RepoResult<Topic<Self::TopicId>> {
        let topic = NewTopicCreated::new(new_topic.name, new_topic.description);
        let result = self
            .db
            .collection::<NewTopicCreated>("topics")
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

    async fn patch(
        &self,
        id: Self::TopicId,
        patch: PatchTopic,
    ) -> OptRepoResult<Topic<Self::TopicId>> {
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
            .collection::<MongoTopic>("topics")
            .find_one_and_update(doc! { "_id": id }, doc! { "$set": update_document })
            .with_options(options)
            .await
            .change_context(TopicRepoError::Patch)
            .map(|t| t.map(From::from))
    }

    async fn delete(&self, id: Self::TopicId) -> OptRepoResult<()> {
        let result = self
            .db
            .collection::<Topic<Self::TopicId>>("topics")
            .delete_one(doc! { "_id": id })
            .await
            .change_context(TopicRepoError::Delete)?;

        Ok((result.deleted_count > 0).then_some(()))
    }
}
