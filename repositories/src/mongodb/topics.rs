use bson::oid::ObjectId;
use bson::{Bson, Document, doc};
use chrono::{DateTime, Utc};
use error_stack::{IntoReport, Report, ResultExt};
use mongodb::options::{FindOneAndUpdateOptions, FindOptions, ReturnDocument};
use mongodb::{Client, Database};
use optional_field::Field;
use serde::{Deserialize, Serialize, Serializer};
use std::fmt::{Display, Formatter};
use tokio_stream::StreamExt;
use topics_core::TopicRepository;
use topics_core::list_filter::TopicListCriteria;
use topics_core::model::{NewTopic, PatchTopic, Topic};
use topics_core::result::{CreateErrorType, OptRepoResult, RepoResult, TopicRepoError};
use tracing::{debug, error, warn};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Eq, Clone, Copy)]
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
    pub fn new_with(id: ObjectId) -> Self {
        Self(id)
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
    fn new(name: String, description: Option<String>, created: DateTime<Utc>) -> Self {
        Self {
            name,
            description,
            created,
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

const TOPICS_DB_NAME: &str = "topics";
const TOPICS_COLLECTION_NAME: &str = "topics";

impl TopicRepo {
    pub fn new(client: Client) -> Self {
        Self {
            db: client.database(TOPICS_DB_NAME),
        }
    }

    pub async fn init(
        connection_details: ConnectionDetails,
    ) -> Result<TopicRepo, Report<ConnectError>> {
        let client = match connection_details {
            ConnectionDetails::Url(url) => Client::with_uri_str(url)
                .await
                .change_context(ConnectError)?,
        };

        Ok(Self {
            db: client.database(TOPICS_DB_NAME),
        })
    }
}

impl TopicRepository for TopicRepo {
    type TopicId = TopicId;

    async fn get(&self, id: Self::TopicId) -> OptRepoResult<Topic<Self::TopicId>> {
        self.db
            .collection::<MongoTopic>(TOPICS_COLLECTION_NAME)
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

        let page_size = if list_criteria.page_size() > i64::MAX as u64 {
            return Err(TopicRepoError::List.into_report())
                .attach_with(|| format!("invalid page_size {}. It is too large and not supported", list_criteria.page_size()));
        } else {
            list_criteria.page_size() as i64
        };

        let options = FindOptions::builder()
            .skip(actual_page)
            .limit(page_size)
            .build();

        self.db
            .collection::<MongoTopic>(TOPICS_COLLECTION_NAME)
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
        let created = Utc::now();
        // block is here to end the borrow of `new_topic` before we create Topic at the end
        let topic = NewTopicCreated::new(new_topic.name, new_topic.description, created);

        let result = self
            .db
            .collection::<NewTopicCreated>(TOPICS_COLLECTION_NAME)
            .insert_one(&topic)
            .await
            .change_context(TopicRepoError::Create(CreateErrorType::DbError))?;

        Ok(Topic {
            id: TopicId::new_with(
                result
                    .inserted_id
                    .as_object_id()
                    .ok_or(TopicRepoError::Create(CreateErrorType::DbError))
                    .attach_with(|| "inserted id for {new_topic:?} was not an ObjectId")?,
            ),
            name: topic.name,
            description: topic.description,
            created,
            updated: None,
        })
    }

    async fn create_many(
        &self,
        new_topics: Vec<NewTopic>,
    ) -> RepoResult<Vec<RepoResult<Topic<Self::TopicId>>>> {
        if new_topics.is_empty() {
            return Ok(vec![]);
        }

        let create_requests = new_topics
            .into_iter()
            .map(|t| NewTopicCreated::new(t.name, t.description, Utc::now()))
            .collect::<Vec<_>>();

        let mut result = self
            .db
            .collection::<NewTopicCreated>(TOPICS_COLLECTION_NAME)
            .insert_many(&create_requests)
            .await
            .change_context(TopicRepoError::Create(CreateErrorType::DbError))?;

        let mut topics = Vec::with_capacity(create_requests.len());

        let mut persisted_topics = 0;

        for (i, create_req) in create_requests.into_iter().enumerate() {
            let Some(id) = result.inserted_ids.remove(&i) else {
                error!("failed to match inserted id to topic status");
                topics.push(
                    Err(TopicRepoError::Create(CreateErrorType::MatchFailure).into_report())
                        as RepoResult<Topic<Self::TopicId>>,
                );
                continue;
            };

            match id.as_object_id() {
                Some(id) => {
                    topics.push(Ok(Topic::new(
                        TopicId(id),
                        create_req.name,
                        create_req.description,
                        create_req.created,
                        None,
                    )));
                    persisted_topics += 1;
                }
                None => {
                    error!("topic was created but the id given back was not an Object ID: {id:?}");
                    topics.push(Err(
                        TopicRepoError::Create(CreateErrorType::MatchFailure).into_report()
                    ));
                }
            }
        }

        debug!("successfully persisted {} new topics", persisted_topics);

        Ok(topics)
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

        if update_document.is_empty() {
            warn!("no topic patch fields specified, returning existing topic");
            return self.get(id).await.change_context(TopicRepoError::Patch);
        }

        update_document.insert("updated", Utc::now().to_rfc3339());

        debug!("Updating document {:?}", update_document);

        let options = FindOneAndUpdateOptions::builder()
            .return_document(ReturnDocument::After)
            .build();

        self.db
            .collection::<MongoTopic>(TOPICS_COLLECTION_NAME)
            .find_one_and_update(doc! { "_id": id }, doc! { "$set": update_document })
            .with_options(options)
            .await
            .change_context(TopicRepoError::Patch)
            .map(|t| t.map(From::from))
    }

    async fn delete(&self, id: Self::TopicId) -> OptRepoResult<()> {
        let result = self
            .db
            .collection::<Topic<Self::TopicId>>(TOPICS_COLLECTION_NAME)
            .delete_one(doc! { "_id": id })
            .await
            .change_context(TopicRepoError::Delete)?;

        Ok((result.deleted_count > 0).then_some(()))
    }
}
