use error_stack::IntoReport;
use indexmap::IndexMap;
use optional_field::Field;
use routing::ArwLock;
use topics_core::{
    TopicRepository,
    list_filter::{TopicFilter, TopicListCriteria},
    model::{NewTopic, PatchTopic, Topic},
    result::{CreateErrorType, OptRepoResult, RepoResult, TopicRepoError},
};

use crate::postgres::topics::TopicId;

mod tests;

#[derive(Clone, Default)]
pub struct InMemoryTopicsRepo {
    db: ArwLock<IndexMap<TopicId, Topic<TopicId>>>,
}

impl TopicRepository for InMemoryTopicsRepo {
    type TopicId = TopicId;

    async fn get(&self, id: Self::TopicId) -> OptRepoResult<Topic<Self::TopicId>> {
        let db = self.db.read().await;

        Ok(db.get(&id).cloned())
    }

    async fn list(
        &self,
        list_criteria: TopicListCriteria,
    ) -> RepoResult<Vec<Topic<Self::TopicId>>> {
        let db = self.db.read().await;

        db.values()
            .skip((list_criteria.page().saturating_sub(1) * list_criteria.page_size()) as usize)
            .filter(|topic| {
                if let Some(filters) = list_criteria.filters() {
                    filters.iter().any(|f| match f {
                        TopicFilter::Name(n) => topic.name.contains(n),
                    })
                } else {
                    true
                }
            })
            .take(list_criteria.page_size() as usize)
            .cloned()
            .map(Ok)
            .collect()
    }

    async fn create(&self, new_topic: NewTopic) -> RepoResult<Topic<Self::TopicId>> {
        let mut db = self.db.write().await;
        let id = TopicId::new();
        let topic = Topic::create(id, new_topic.name, new_topic.description);
        db.insert(id, topic.clone());

        Ok(topic)
    }

    async fn create_many(
        &self,
        topics: Vec<NewTopic>,
    ) -> RepoResult<Vec<RepoResult<Topic<Self::TopicId>>>> {
        let mut db = self.db.write().await;
        Ok(topics
            .into_iter()
            .map(|t| Topic::create(TopicId::new(), t.name, t.description))
            .map(|topic| {
                db.insert(topic.id, topic.clone());
                Ok(topic)
            })
            .collect())
    }

    async fn patch(
        &self,
        id: Self::TopicId,
        patch: PatchTopic,
    ) -> OptRepoResult<Topic<Self::TopicId>> {
        let mut db = self.db.write().await;

        Ok(db.get_mut(&id).map(|topic| {
            if let Some(name) = patch.name {
                topic.name = name;
            }

            if let Field::Present(desc) = patch.description {
                topic.description = desc
            }
            topic.clone()
        }))
    }

    async fn delete(&self, id: Self::TopicId) -> OptRepoResult<()> {
        let mut db = self.db.write().await;
        Ok(db.shift_remove(&id).map(|_| ()))
    }
}

#[derive(Clone)]
pub struct FailingTopicsRepo {
    create_err_reason: CreateErrorType,
}

impl Default for FailingTopicsRepo {
    fn default() -> Self {
        Self {
            create_err_reason: CreateErrorType::DbError,
        }
    }
}

impl TopicRepository for FailingTopicsRepo {
    type TopicId = TopicId;

    async fn get(&self, _: Self::TopicId) -> OptRepoResult<Topic<Self::TopicId>> {
        Err(TopicRepoError::Get.into_report())
    }

    async fn list(&self, _: TopicListCriteria) -> RepoResult<Vec<Topic<Self::TopicId>>> {
        Err(TopicRepoError::List.into_report())
    }

    async fn create(&self, _: NewTopic) -> RepoResult<Topic<Self::TopicId>> {
        Err(TopicRepoError::Create(self.create_err_reason.clone()).into_report())
    }

    async fn create_many(
        &self,
        _: Vec<NewTopic>,
    ) -> RepoResult<Vec<RepoResult<Topic<Self::TopicId>>>> {
        Err(TopicRepoError::Create(self.create_err_reason.clone()).into_report())
    }

    async fn patch(&self, _: Self::TopicId, _: PatchTopic) -> OptRepoResult<Topic<Self::TopicId>> {
        Err(TopicRepoError::Patch.into_report())
    }

    async fn delete(&self, _: Self::TopicId) -> OptRepoResult<()> {
        Err(TopicRepoError::Delete.into_report())
    }
}
