use topics_core::{
    TopicEngine, TopicRepository,
    list_filter::TopicListCriteria,
    model::{NewTopic, PatchTopic, Topic},
    result::{OptRepoResult, RepoResult, TopicRepoError},
};

type Id = i32;

use crate::service::TopicService;

#[derive(Clone)]
struct MockRepo {
    get: fn(Id) -> OptRepoResult<Topic<Id>>,
    list: fn(TopicListCriteria) -> RepoResult<Vec<Topic<Id>>>,
    create: fn(NewTopic) -> RepoResult<Topic<Id>>,
    create_many: fn(Vec<NewTopic>) -> RepoResult<Vec<RepoResult<Topic<Id>>>>,
    patch: fn(Id, PatchTopic) -> OptRepoResult<Topic<Id>>,
    delete: fn(Id) -> OptRepoResult<()>,
}

impl Default for MockRepo {
    fn default() -> Self {
        Self {
            get: |_| panic!("'get' called unexpectedly"),
            list: |_| panic!("'list' called unexpectedly"),
            create: |_| panic!("'create' called unexpectdly"),
            create_many: |_| panic!("'create_many' called unexpectedly"),
            patch: |_, _| panic!("'patch' called unexpectedly"),
            delete: |_| panic!("'delete' called unexpectedly"),
        }
    }
}

impl TopicRepository for MockRepo {
    type TopicId = Id;

    fn get(
        &self,
        id: Self::TopicId,
    ) -> impl Future<Output = OptRepoResult<Topic<Self::TopicId>>> + Send {
        let val = (self.get)(id);
        async move { val }
    }

    fn list(
        &self,
        list_criteria: TopicListCriteria,
    ) -> impl Future<Output = RepoResult<Vec<Topic<Self::TopicId>>>> + Send {
        let val = (self.list)(list_criteria);
        async move { val }
    }

    fn create(
        &self,
        new_topic: NewTopic,
    ) -> impl Future<Output = RepoResult<Topic<Self::TopicId>>> + Send {
        let val = (self.create)(new_topic);
        async move { val }
    }

    fn create_many(
        &self,
        topics: Vec<NewTopic>,
    ) -> impl Future<Output = RepoResult<Vec<RepoResult<Topic<Self::TopicId>>>>> + Send {
        let val = (self.create_many)(topics);
        async move { val }
    }

    fn patch(
        &self,
        id: Self::TopicId,
        patch: PatchTopic,
    ) -> impl Future<Output = OptRepoResult<Topic<Self::TopicId>>> + Send {
        let val = (self.patch)(id, patch);
        async move { val }
    }

    fn delete(&self, id: Self::TopicId) -> impl Future<Output = OptRepoResult<()>> + Send {
        let val = (self.delete)(id);
        async move { val }
    }
}

#[derive(Clone)]
struct TestEngine(MockRepo);
impl TopicEngine for TestEngine {
    type TopicId = Id;

    type Repo = MockRepo;

    fn repo(&self) -> Self::Repo {
        self.0.clone()
    }
}

#[tokio::test]
async fn get_repo_returns_none_service_returns_none() {
    let service = TopicService::new(TestEngine(MockRepo {
        get: |_| Ok(None),
        ..Default::default()
    }));

    assert!(service.get(1).await.unwrap().is_none());
}
