use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use mongodb::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NewTopicCreated {
    pub name: String,
    pub description: Option<String>,
    pub created: DateTime<Utc>,
}

impl NewTopicCreated {
    pub fn new(name: impl Into<String>, description: Option<impl Into<String>>) -> Self {
        Self {
            name: name.into(),
            description: description.map(|s| s.into()),
            created: Utc::now(),
        }
    }
}

#[derive(Default)]
pub struct Migration {
    steps: Vec<MigrationStep>,
    total: usize,
}

enum MigrationStep {
    Fill(usize),
    FillWith {
        fill: usize,
        name: String,
        description: Option<String>,
    },
    Topic(NewTopicCreated),
    Topics(Vec<NewTopicCreated>),
}

impl Migration {
    /// Use this if you need to insert `fill` amount of topics, but won't be relying on their contents in your test
    pub fn fill(mut self, fill: usize) -> Self {
        self.steps.push(MigrationStep::Fill(fill));
        self.total += fill;
        self
    }

    /// Use this if you want to fill in `fill` amount of topics using a specific name and/or description.
    /// Their contents can reliably be tested against
    pub fn fill_with(
        mut self,
        fill: usize,
        name: impl Into<String>,
        description: Option<impl Into<String>>,
    ) -> Self {
        self.total += fill;
        self.steps.push(MigrationStep::FillWith {
            fill,
            name: name.into(),
            description: description.map(Into::into),
        });
        self
    }

    pub fn single(mut self, topic: NewTopicCreated) -> Self {
        self.total += 1;
        self.steps.push(MigrationStep::Topic(topic));
        self
    }

    pub fn multi<I>(mut self, topics: I) -> Self
    where
        I: IntoIterator<Item = NewTopicCreated>,
    {
        let topics = topics.into_iter().collect::<Vec<_>>();
        self.total += topics.len();
        self.steps.push(MigrationStep::Topics(topics));
        self
    }

    pub async fn run(self, client: Client) -> Vec<ObjectId> {
        let mut topics = Vec::with_capacity(self.total);

        for step in self.steps {
            match step {
                MigrationStep::Fill(fill) => {
                    topics.extend((0..fill).map(|_| generate_filler_topic()))
                }
                MigrationStep::FillWith {
                    fill,
                    name,
                    description,
                } => {
                    let iter = (0..fill).map(|_| NewTopicCreated::new(&name, description.as_ref()));
                    topics.extend(iter)
                }
                MigrationStep::Topic(new_topic) => topics.extend([new_topic]),
                MigrationStep::Topics(new_topics) => topics.extend(new_topics),
            }
        }

        insert_many(client, topics).await
    }
}

fn generate_filler_topic() -> NewTopicCreated {
    NewTopicCreated::new("filler topic", Some("filler description"))
}

async fn insert_many<I>(client: Client, topics: I) -> Vec<ObjectId>
where
    I: IntoIterator<Item = NewTopicCreated>,
{
    let ids = client
        .database("topics")
        .collection::<NewTopicCreated>("topics")
        .insert_many(topics)
        .await
        .unwrap()
        .inserted_ids;

    (0..ids.len())
        .map(|i| ids[&i].as_object_id().unwrap())
        .collect()
}
