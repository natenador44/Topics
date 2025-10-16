use chrono::{DateTime, Utc};
use mongodb::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NewTopicCreated {
    name: String,
    description: Option<String>,
    created: DateTime<Utc>,
}

impl NewTopicCreated {
    pub fn new(name: &str, description: Option<&str>) -> Self {
        Self {
            name: name.into(),
            description: description.map(|s| s.to_string()),
            created: Utc::now(),
        }
    }
}

pub async fn sandwich<I>(client: Client, start_fill: usize, middle: I, end_fill: usize)
where
    I: IntoIterator<Item = NewTopicCreated> + Send + Sync + 'static,
{
    let topics = (0..start_fill)
        .map(|_| NewTopicCreated::new("start fill", Some("start fill desc")))
        .chain(middle)
        .chain((0..end_fill).map(|_| NewTopicCreated::new("end fill", Some("end fill desc"))));

    insert_many(client, topics).await
}

pub async fn start_with_and_fill<I>(client: Client, start_with: I, fill: usize)
where
    I: IntoIterator<Item = NewTopicCreated> + Send + Sync + 'static,
{
    let topics = start_with
        .into_iter()
        .chain((0..fill).map(|_| NewTopicCreated::new("new topic", Some("new description"))));

    insert_many(client, topics).await;
}

pub async fn fill(client: Client, num: usize) {
    let topics = (0..num).map(|_| NewTopicCreated::new("new topic", Some("new desc")));

    insert_many(client, topics).await;
}

async fn insert_many<I>(client: Client, topics: I)
where
    I: IntoIterator<Item = NewTopicCreated>,
{
    client
        .database("topics")
        .collection::<NewTopicCreated>("topics")
        .insert_many(topics)
        .await
        .unwrap();
}
