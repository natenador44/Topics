use engine::Pagination;
use optional_field::Field;
use rstest::rstest;
use testcontainers_modules::testcontainers::{ContainerAsync, Image};
use topics_core::TopicRepository;
use topics_core::list_filter::TopicListCriteria;
use topics_core::model::{NewTopic, PatchTopic};
const DEFAULT_PAGINATION: Pagination = Pagination {
    page: 1,
    page_size: None,
};
const DEFAULT_PAGE_SIZE: u64 = 25;

#[rstest]
#[case::mongo(mongo::runtime())]
#[case::postgres(postgres::runtime())]
#[tokio::test]
async fn get_no_data_returns_none<C, R>(
    #[future(awt)]
    #[case]
    runtime: TestRuntime<C, R>,
) where
    C: Image,
    R: TopicRepository,
{
    let repo = &runtime.repo;

    let result = repo.get(runtime.generate_new_id()).await.unwrap();

    assert!(result.is_none());
}

#[rstest]
#[case::mongo(mongo::runtime())]
#[case::postgres(postgres::runtime())]
#[tokio::test]
async fn create_then_get_returns_created_topic<C, R>(
    #[future(awt)]
    #[case]
    runtime: TestRuntime<C, R>,
) where
    C: Image,
    R: TopicRepository,
{
    let repo = runtime.repo;

    let created = repo.create(default_new_topic()).await.unwrap();

    let found = repo
        .get(created.id)
        .await
        .unwrap()
        .expect("recently created topic exists");

    assert_eq!(&created, &found);
}

#[rstest]
#[case::mongo(mongo::runtime())]
#[case::postgres(postgres::runtime())]
#[tokio::test]
async fn no_topics_created_list_returns_empty_vec<C, R>(
    #[future(awt)]
    #[case]
    runtime: TestRuntime<C, R>,
) where
    C: Image,
    R: TopicRepository,
{
    let repo = runtime.repo;

    let topics = repo.list(default_list_criteria()).await.unwrap();

    assert!(topics.is_empty());
}

#[rstest]
#[case::mongo(mongo::runtime())]
#[case::postgres(postgres::runtime())]
#[tokio::test]
async fn create_single_then_list_returns_single_result<C, R>(
    #[future(awt)]
    #[case]
    runtime: TestRuntime<C, R>,
) where
    C: Image,
    R: TopicRepository,
{
    let repo = runtime.repo;

    let created = repo.create(default_new_topic()).await.unwrap();

    let listed = repo.list(default_list_criteria()).await.unwrap();

    assert_eq!(1, listed.len());
    assert_eq!(&created, listed.get(0).unwrap());
}

#[rstest]
#[case::mongo(mongo::runtime())]
#[case::postgres(postgres::runtime())]
#[tokio::test]
async fn list_page_1_or_0_returns_first_page<C, R>(
    #[future(awt)]
    #[case]
    runtime: TestRuntime<C, R>,
) where
    C: Image,
    R: TopicRepository,
{
    let repo = runtime.repo;

    let created = repo.create(default_new_topic()).await.unwrap();

    let listed = repo
        .list(TopicListCriteria::new(
            Pagination::with_default_page_size(1),
            DEFAULT_PAGE_SIZE,
        ))
        .await
        .unwrap();

    assert_eq!(1, listed.len());
    assert_eq!(&created, listed.get(0).unwrap(), "page = 1");

    let listed = repo
        .list(TopicListCriteria::new(
            Pagination::with_default_page_size(0),
            DEFAULT_PAGE_SIZE,
        ))
        .await
        .unwrap();

    assert_eq!(1, listed.len());
    assert_eq!(&created, listed.get(0).unwrap(), "page = 0");
}

#[rstest]
#[case::mongo(mongo::runtime())]
#[case::postgres(postgres::runtime())]
#[tokio::test]
async fn list_returns_all_created_if_create_called_n_lt_page_size_times<C, R>(
    #[future(awt)]
    #[case]
    runtime: TestRuntime<C, R>,
) where
    C: Image,
    R: TopicRepository,
{
    let repo = runtime.repo;

    let mut created = Vec::with_capacity(10);

    for _ in 0..10 {
        created.push(repo.create(default_new_topic()).await.unwrap());
    }

    let listed = repo.list(default_list_criteria()).await.unwrap();

    assert_eq!(10, listed.len());
    for (i, expected) in created.into_iter().enumerate() {
        assert_eq!(&expected, listed.get(i).unwrap(), "topic index {i}");
    }
}

#[rstest]
#[case::mongo(mongo::runtime())]
#[case::postgres(postgres::runtime())]
#[tokio::test]
async fn list_returns_error_if_page_gt_i64_max<C, R>(
    #[future(awt)]
    #[case]
    runtime: TestRuntime<C, R>,
) where
    C: Image,
    R: TopicRepository,
{
    let repo = runtime.repo;

    let result = repo
        .list(TopicListCriteria::new(
            Pagination {
                page: i64::MAX as u64 + 1,
                page_size: None,
            },
            DEFAULT_PAGE_SIZE,
        ))
        .await;

    assert!(result.is_err());
}

#[rstest]
#[case::mongo(mongo::runtime())]
#[case::postgres(postgres::runtime())]
#[tokio::test]
async fn list_returns_error_if_page_size_gt_i64_max<C, R>(
    #[future(awt)]
    #[case]
    runtime: TestRuntime<C, R>,
) where
    C: Image,
    R: TopicRepository,
{
    let repo = runtime.repo;

    let result = repo
        .list(TopicListCriteria::new(
            Pagination {
                page: 1,
                page_size: Some(i64::MAX as u64 + 1),
            },
            DEFAULT_PAGE_SIZE,
        ))
        .await;

    assert!(result.is_err());
}

#[rstest]
#[case::mongo(mongo::runtime())]
#[case::postgres(postgres::runtime())]
#[tokio::test]
async fn list_returns_max_page_size_results<C, R>(
    #[future(awt)]
    #[case]
    runtime: TestRuntime<C, R>,
) where
    C: Image,
    R: TopicRepository,
{
    let repo = runtime.repo;

    let new_topics = (0..100)
        .map(|i| NewTopic::new(format!("topic{i}"), Some(format!("topic{i} desc"))))
        .collect::<Vec<_>>();

    repo.create_many(new_topics.clone()).await.unwrap();

    let topics = repo.list(default_list_criteria()).await.unwrap();

    assert_eq!(DEFAULT_PAGE_SIZE as usize, topics.len());

    for (i, new_topic) in new_topics
        .into_iter()
        .take(DEFAULT_PAGE_SIZE as usize)
        .enumerate()
    {
        let topic = &topics[i];
        assert_eq!(&new_topic.name, &topic.name);
        assert_eq!(&new_topic.description, &topic.description);
    }
}

#[rstest]
#[case::mongo(mongo::runtime())]
#[case::postgres(postgres::runtime())]
#[tokio::test]
async fn create_many_empty_vec_returns_empty_vec<C, R>(
    #[future(awt)]
    #[case]
    runtime: TestRuntime<C, R>,
) where
    C: Image,
    R: TopicRepository,
{
    let repo = runtime.repo;

    let created = repo.create_many(Vec::new()).await.unwrap();
    assert!(created.is_empty());
}

#[rstest]
#[case::mongo(mongo::runtime())]
#[case::postgres(postgres::runtime())]
#[tokio::test]
async fn create_many_single_is_created<C, R>(
    #[future(awt)]
    #[case]
    runtime: TestRuntime<C, R>,
) where
    C: Image,
    R: TopicRepository,
{
    let repo = runtime.repo;

    let original = vec![NewTopic::new("blah", Some("blah desc"))];

    let updated = repo.create_many(original.clone()).await.unwrap();

    assert_eq!(1, updated.len());
    let topic = updated[0].as_ref().expect("topic create was successful");

    assert_eq!("blah", &topic.name);
    assert_eq!(Some("blah desc"), topic.description.as_deref());

    let created_topic = repo
        .get(topic.id)
        .await
        .unwrap()
        .expect("created topic is in db");

    assert_eq!(topic, &created_topic);
}

#[rstest]
#[case::mongo(mongo::runtime())]
#[case::postgres(postgres::runtime())]
#[tokio::test]
async fn create_many_multi_pending_is_created<C, R>(
    #[future(awt)]
    #[case]
    runtime: TestRuntime<C, R>,
) where
    C: Image,
    R: TopicRepository,
{
    let repo = runtime.repo;

    let new_topics = vec![
        NewTopic::new("topic1", Some("topic1 desc")),
        NewTopic::new("topic2", Some("topic2 desc")),
        NewTopic::new("topic3", Some("topic3 desc")),
        NewTopic::new("topic4", Some("topic4 desc")),
        NewTopic::new("topic5", Some("topic5 desc")),
        NewTopic::new("topic6", Some("topic6 desc")),
    ];

    let created = repo.create_many(new_topics.clone()).await.unwrap();

    assert_eq!(new_topics.len(), created.len());

    for (i, topic_req) in new_topics.into_iter().enumerate() {
        let created_topic = created[i].as_ref().expect("topic created");

        assert_eq!(topic_req.name, created_topic.name);
        assert_eq!(topic_req.description, created_topic.description);

        let queried_topic = repo
            .get(created_topic.id)
            .await
            .unwrap()
            .expect("topic was created in db");

        assert_eq!(created_topic, &queried_topic);
    }
}

#[rstest]
#[case::mongo(mongo::runtime())]
#[case::postgres(postgres::runtime())]
#[tokio::test]
async fn patch_name_update<C, R>(
    #[future(awt)]
    #[case]
    runtime: TestRuntime<C, R>,
) where
    C: Image,
    R: TopicRepository,
{
    let repo = runtime.repo;

    let created_topic = repo
        .create(NewTopic::new("topic1", Some("topic1 desc")))
        .await
        .unwrap();

    let updated_topic = repo
        .patch(
            created_topic.id,
            PatchTopic::new(Some("topic2".into()), Field::Missing),
        )
        .await
        .unwrap()
        .expect("topic should have been found");

    assert_eq!(created_topic.id, updated_topic.id);
    assert_eq!("topic2", &updated_topic.name);
    assert_eq!(created_topic.description, updated_topic.description);
    assert_eq!(created_topic.created, updated_topic.created);
    assert!(updated_topic.updated.is_some());
}

#[rstest]
#[case::mongo(mongo::runtime())]
#[case::postgres(postgres::runtime())]
#[tokio::test]
async fn patch_name_desc_update<C, R>(
    #[future(awt)]
    #[case]
    runtime: TestRuntime<C, R>,
) where
    C: Image,
    R: TopicRepository,
{
    let repo = runtime.repo;

    let created_topic = repo
        .create(NewTopic::new("topic1", Some("topic1 desc")))
        .await
        .unwrap();

    let updated_topic = repo
        .patch(
            created_topic.id,
            PatchTopic::new(
                Some("topic2".into()),
                Field::Present(Some("topic2 desc".into())),
            ),
        )
        .await
        .unwrap()
        .expect("topic should have been found");

    assert_eq!(created_topic.id, updated_topic.id);
    assert_eq!("topic2", &updated_topic.name);
    assert_eq!(Some("topic2 desc"), updated_topic.description.as_deref());
    assert_eq!(created_topic.created, updated_topic.created);
    assert!(updated_topic.updated.is_some());
}

#[rstest]
#[case::mongo(mongo::runtime())]
#[case::postgres(postgres::runtime())]
#[tokio::test]
async fn patch_non_null_desc_update<C, R>(
    #[future(awt)]
    #[case]
    runtime: TestRuntime<C, R>,
) where
    C: Image,
    R: TopicRepository,
{
    let repo = runtime.repo;

    let created_topic = repo
        .create(NewTopic::new("topic1", Some("topic1 desc")))
        .await
        .unwrap();

    let updated_topic = repo
        .patch(
            created_topic.id,
            PatchTopic::new(None, Field::Present(Some("topic2 desc".into()))),
        )
        .await
        .unwrap()
        .expect("topic should have been found");

    assert_eq!(created_topic.id, updated_topic.id);
    assert_eq!(&created_topic.name, &updated_topic.name);
    assert_eq!(Some("topic2 desc"), updated_topic.description.as_deref());
    assert_eq!(created_topic.created, updated_topic.created);
    assert!(updated_topic.updated.is_some());
}

#[rstest]
#[case::mongo(mongo::runtime())]
#[case::postgres(postgres::runtime())]
#[tokio::test]
async fn patch_null_desc_update<C, R>(
    #[future(awt)]
    #[case]
    runtime: TestRuntime<C, R>,
) where
    C: Image,
    R: TopicRepository,
{
    let repo = runtime.repo;

    let created_topic = repo
        .create(NewTopic::new("topic1", Some("topic1 desc")))
        .await
        .unwrap();

    let updated_topic = repo
        .patch(
            created_topic.id,
            PatchTopic::new(None, Field::Present(None)),
        )
        .await
        .unwrap()
        .expect("topic should have been found");

    assert_eq!(created_topic.id, updated_topic.id);
    assert_eq!(&created_topic.name, &updated_topic.name);
    assert_eq!(None, updated_topic.description.as_deref());
    assert_eq!(created_topic.created, updated_topic.created);
    assert!(updated_topic.updated.is_some());
}

#[rstest]
#[case::mongo(mongo::runtime())]
#[case::postgres(postgres::runtime())]
#[tokio::test]
async fn patch_no_updates_leaves_topic_alone_returns_existing_topic<C, R>(
    #[future(awt)]
    #[case]
    runtime: TestRuntime<C, R>,
) where
    C: Image,
    R: TopicRepository,
{
    let repo = runtime.repo;

    let created_topic = repo
        .create(NewTopic::new("topic1", Some("topic1 desc")))
        .await
        .unwrap();

    let updated_topic = repo
        .patch(created_topic.id, PatchTopic::new(None, Field::Missing))
        .await
        .unwrap()
        .expect("topic should have been found");

    assert_eq!(created_topic, updated_topic);
}

#[rstest]
#[case::mongo(mongo::runtime())]
#[case::postgres(postgres::runtime())]
#[tokio::test]
async fn patch_no_created_topics_returns_none<C, R>(
    #[future(awt)]
    #[case]
    runtime: TestRuntime<C, R>,
) where
    C: Image,
    R: TopicRepository,
{
    let repo = &runtime.repo;

    let updated_topic = repo
        .patch(
            runtime.generate_new_id(),
            PatchTopic::new(None, Field::Present(None)),
        )
        .await
        .unwrap();

    assert!(updated_topic.is_none());
}

#[rstest]
#[case::mongo(mongo::runtime())]
#[case::postgres(postgres::runtime())]
#[tokio::test]
async fn patch_topic_not_found_returns_none<C, R>(
    #[future(awt)]
    #[case]
    runtime: TestRuntime<C, R>,
) where
    C: Image,
    R: TopicRepository,
{
    let repo = &runtime.repo;

    let _ = repo
        .create(NewTopic::new("topic1", Some("topic1 desc")))
        .await
        .unwrap();

    let updated_topic = repo
        .patch(
            runtime.generate_new_id(),
            PatchTopic::new(None, Field::Missing),
        )
        .await
        .unwrap();

    assert!(updated_topic.is_none());
}

#[rstest]
#[case::mongo(mongo::runtime())]
#[case::postgres(postgres::runtime())]
#[tokio::test]
async fn delete_no_topics_created_returns_none<C, R>(
    #[future(awt)]
    #[case]
    runtime: TestRuntime<C, R>,
) where
    C: Image,
    R: TopicRepository,
{
    let repo = &runtime.repo;

    let result = repo
        .delete(runtime.generate_new_id())
        .await
        .expect("topic delete should not fail");

    assert!(result.is_none());
}

#[rstest]
#[case::mongo(mongo::runtime())]
#[case::postgres(postgres::runtime())]
#[tokio::test]
async fn delete_no_matching_existing_topics_returns_none<C, R>(
    #[future(awt)]
    #[case]
    runtime: TestRuntime<C, R>,
) where
    C: Image,
    R: TopicRepository,
{
    let repo = &runtime.repo;

    let topic = repo
        .create(NewTopic::new("topic1", Some("topic1 desc")))
        .await
        .unwrap();

    let result = repo
        .delete(runtime.generate_new_id())
        .await
        .expect("topic delete should not fail");

    assert!(result.is_none());

    let topic = repo.get(topic.id).await.unwrap();

    assert!(topic.is_some());
}

#[rstest]
#[case::mongo(mongo::runtime())]
#[case::postgres(postgres::runtime())]
#[tokio::test]
async fn delete_with_existing_topic_deletes_topic<C, R>(
    #[future(awt)]
    #[case]
    runtime: TestRuntime<C, R>,
) where
    C: Image,
    R: TopicRepository,
{
    let repo = runtime.repo;

    let topic = repo
        .create(NewTopic::new("topic1", Some("topic1 desc")))
        .await
        .unwrap();

    let result = repo
        .delete(topic.id)
        .await
        .expect("topic delete should not fail");

    assert!(result.is_some());

    let topic = repo.get(topic.id).await.unwrap();

    assert!(topic.is_none());
}

pub fn default_new_topic() -> NewTopic {
    NewTopic::new("test topic 1", Some("test topic 1 description"))
}

fn default_list_criteria() -> TopicListCriteria {
    TopicListCriteria::new(DEFAULT_PAGINATION, DEFAULT_PAGE_SIZE)
}

struct TestRuntime<C, R>
where
    C: Image,
    R: TopicRepository,
{
    _container: ContainerAsync<C>,
    repo: R,
    new_id_fn: Box<dyn Fn() -> R::TopicId>,
}

impl<C, R> TestRuntime<C, R>
where
    C: Image,
    R: TopicRepository,
{
    fn new<F>(container: ContainerAsync<C>, repo: R, new_id_fn: F) -> Self
    where
        F: Fn() -> R::TopicId + 'static,
    {
        Self {
            _container: container,
            repo,
            new_id_fn: Box::new(new_id_fn),
        }
    }

    fn generate_new_id(&self) -> R::TopicId {
        (self.new_id_fn)()
    }
}

mod mongo {
    use crate::topics::TestRuntime;
    use bson::oid::ObjectId;
    use repositories::mongodb::topics as mongo_repo;
    use testcontainers_modules::mongo::Mongo;
    use testcontainers_modules::testcontainers::ContainerAsync;
    use testcontainers_modules::testcontainers::runners::AsyncRunner;

    pub async fn runtime() -> TestRuntime<Mongo, mongo_repo::TopicRepo> {
        let mongo_container = container().await;
        let host = mongo_container.get_host().await.unwrap();
        let port = mongo_container.get_host_port_ipv4(27017).await.unwrap();

        let repo = mongo_repo::TopicRepo::init(mongo_repo::ConnectionDetails::Url(format!(
            "mongodb://{host}:{port}/?authSource=admin"
        )))
        .await
        .unwrap();

        TestRuntime::new(mongo_container, repo, || {
            mongo_repo::TopicId::new_with(ObjectId::new())
        })
    }

    async fn container() -> ContainerAsync<Mongo> {
        Mongo::default().start().await.unwrap()
    }
}

mod postgres {
    use crate::topics::TestRuntime;
    use repositories::postgres::ConnectionDetails;
    use repositories::postgres::initializer::RepoInitializer;
    use repositories::postgres::topics as postgres_repo;
    use repositories::postgres::topics::TopicId;
    use testcontainers_modules::postgres::Postgres;

    pub async fn runtime() -> TestRuntime<Postgres, postgres_repo::TopicRepo> {
        let container = crate::postgres::container().await;
        let host = container.get_host().await.unwrap();
        let port = container.get_host_port_ipv4(5432).await.unwrap();

        let repo = RepoInitializer::new()
            .with_topics()
            .init(
                ConnectionDetails::Url(format!(
                    "postgresql://testuser:testpass@{host}:{port}/topics"
                )),
                Some(1),
            )
            .await
            .unwrap();

        TestRuntime::new(container, repo, || TopicId::new())
    }
}
