use engine::id::Id;
use rstest::rstest;
use sets_core::SetRepository;
use testcontainers_modules::testcontainers::{ContainerAsync, Image};
use topics_core::TopicRepository;
use topics_core::model::NewTopic;
#[rstest]
#[case::postgres(postgres::runtime())]
#[tokio::test]
async fn get_no_set_data_returns_none<C, R>(
    #[future(awt)]
    #[case]
    runtime: TestRuntime<C, R>,
) where
    C: Image,
    R: Repos,
{
    let topics = &runtime.repos.topics();

    let new_topic = topics
        .create(NewTopic::new("topic1".into(), None))
        .await
        .expect("topic creation works");

    let sets = &runtime.repos.sets();

    let set = sets
        .get(runtime.existing_topic_set_key(new_topic.id))
        .await
        .expect("set get success");

    assert!(set.is_none());
}

mod postgres {
    use crate::sets::{Repos, TestRuntime};
    use repositories::postgres::ConnectionDetails;
    use repositories::postgres::initializer::RepoInitializer;
    use repositories::postgres::sets::{PostgresSetKey, SetId, SetRepo};
    use repositories::postgres::topics::{TopicId, TopicRepo};
    use testcontainers_modules::postgres::Postgres;

    pub struct PostgresRepos {
        topics: TopicRepo,
        sets: SetRepo,
    }

    impl Repos for PostgresRepos {
        type TopicId = TopicId;
        type SetId = SetId;
        type SetKey = PostgresSetKey;
        type Topic = TopicRepo;
        type Set = SetRepo;

        fn topics(&self) -> Self::Topic {
            self.topics.clone()
        }

        fn sets(&self) -> Self::Set {
            self.sets.clone()
        }
    }

    pub async fn runtime() -> TestRuntime<Postgres, PostgresRepos> {
        let container = crate::postgres::container().await;
        let host = container.get_host().await.unwrap();
        let port = container.get_host_port_ipv4(5432).await.unwrap();

        let connection_details = ConnectionDetails::Url(format!(
            "postgresql://testuser:testpass@{host}:{port}/topics"
        ));

        let (topics, sets) = RepoInitializer::new()
            .with_sets()
            .init(connection_details, Some(1))
            .await
            .expect("repo initialization success");

        TestRuntime::new(container, PostgresRepos { topics, sets }, generate_set_key)
    }

    fn generate_set_key(topic_id: Option<TopicId>, set_id: Option<SetId>) -> PostgresSetKey {
        match (topic_id, set_id) {
            (Some(t), Some(s)) => PostgresSetKey(t, s),
            (Some(t), None) => PostgresSetKey(t, SetId::new()),
            (None, Some(s)) => PostgresSetKey(TopicId::new(), s),
            (None, None) => PostgresSetKey(TopicId::new(), SetId::new()),
        }
    }
}

trait Repos {
    type TopicId: Id;
    type SetId: Id;

    type SetKey: sets_core::SetKey<TopicId = Self::TopicId, SetId = Self::SetId>;
    type Topic: TopicRepository<TopicId = Self::TopicId>;
    type Set: SetRepository<SetKey = Self::SetKey>;

    fn topics(&self) -> Self::Topic;
    fn sets(&self) -> Self::Set;
}

type SetKeyFn<T, S, K> = Box<dyn Fn(Option<T>, Option<S>) -> K>;

struct TestRuntime<C, R>
where
    C: Image,
    R: Repos,
{
    _container: ContainerAsync<C>,
    repos: R,
    set_key_gen: SetKeyFn<R::TopicId, R::SetId, R::SetKey>,
}

impl<C, R> TestRuntime<C, R>
where
    C: Image,
    R: Repos,
{
    fn new<F>(container: ContainerAsync<C>, repos: R, set_key_gen: F) -> Self
    where
        F: Fn(Option<R::TopicId>, Option<R::SetId>) -> R::SetKey + 'static,
    {
        Self {
            _container: container,
            repos,
            set_key_gen: Box::new(set_key_gen),
        }
    }

    fn existing_topic_set_key(
        &self,
        topic_id: <R::Topic as TopicRepository>::TopicId,
    ) -> R::SetKey {
        (self.set_key_gen)(Some(topic_id), None)
    }
}
