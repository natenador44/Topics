use engine::Pagination;
use optional_field::Field;
use repositories::postgres::topics::{ConnectionDetails, TopicId, TopicRepo};
use rstest::{fixture, rstest};
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{ContainerAsync, runners::AsyncRunner},
};
use topics_core::TopicRepository;
use topics_core::list_filter::TopicListCriteria;
use topics_core::model::{NewTopic, PatchTopic};

struct TestRuntime {
    _container: ContainerAsync<Postgres>,
    repo: TopicRepo,
}
const DEFAULT_PAGINATION: Pagination = Pagination {
    page: 1,
    page_size: None,
};
const DEFAULT_PAGE_SIZE: u64 = 25;

fn default_list_criteria() -> TopicListCriteria {
    TopicListCriteria::new(DEFAULT_PAGINATION, DEFAULT_PAGE_SIZE)
}

fn default_new_topic() -> NewTopic {
    NewTopic::new(
        "test topic 1".into(),
        Some("test topic 1 description".into()),
    )
}

#[rstest]
#[tokio::test]
async fn get_no_data_returns_none(#[future] runtime: TestRuntime) {
    let runtime = runtime.await;
    let repo = runtime.repo;

    let result = repo.get(TopicId::new()).await.unwrap();

    assert!(result.is_none());
}

#[rstest]
#[tokio::test]
async fn create_then_get_returns_created_topic(#[future] runtime: TestRuntime) {
    let runtime = runtime.await;
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
#[tokio::test]
async fn no_topics_created_list_returns_empty_vec(#[future] runtime: TestRuntime) {
    let runtime = runtime.await;
    let repo = runtime.repo;

    let topics = repo.list(default_list_criteria()).await.unwrap();

    assert!(topics.is_empty());
}

#[rstest]
#[tokio::test]
async fn create_single_then_list_returns_single_result(#[future] runtime: TestRuntime) {
    let runtime = runtime.await;
    let repo = runtime.repo;

    let created = repo.create(default_new_topic()).await.unwrap();

    let listed = repo.list(default_list_criteria()).await.unwrap();

    assert_eq!(1, listed.len());
    assert_eq!(&created, listed.get(0).unwrap());
}

#[rstest]
#[tokio::test]
async fn list_page_1_or_0_returns_first_page(#[future] runtime: TestRuntime) {
    let runtime = runtime.await;
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
#[tokio::test]
async fn list_returns_all_created_if_create_called_n_lt_page_size_times(
    #[future] runtime: TestRuntime,
) {
    let runtime = runtime.await;
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
#[tokio::test]
async fn list_returns_error_if_page_gt_i64_max(#[future] runtime: TestRuntime) {
    let runtime = runtime.await;
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
#[tokio::test]
async fn list_returns_error_if_page_size_gt_i64_max(#[future] runtime: TestRuntime) {
    let runtime = runtime.await;
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
#[tokio::test]
async fn list_returns_max_page_size_results(#[future] runtime: TestRuntime) {
    let runtime = runtime.await;
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
#[tokio::test]
async fn create_many_empty_vec_returns_empty_vec(#[future] runtime: TestRuntime) {
    let runtime = runtime.await;
    let repo = runtime.repo;

    let created = repo.create_many(Vec::new()).await.unwrap();
    assert!(created.is_empty());
}

#[rstest]
#[tokio::test]
async fn create_many_single_is_created(#[future] runtime: TestRuntime) {
    let runtime = runtime.await;
    let repo = runtime.repo;

    let original = vec![NewTopic::new("blah".into(), Some("blah desc".into()))];

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
#[tokio::test]
async fn create_many_multi_pending_is_created(#[future] runtime: TestRuntime) {
    let runtime = runtime.await;
    let repo = runtime.repo;

    let new_topics = vec![
        NewTopic::new("topic1".into(), Some("topic1 desc".into())),
        NewTopic::new("topic2".into(), Some("topic2 desc".into())),
        NewTopic::new("topic3".into(), Some("topic3 desc".into())),
        NewTopic::new("topic4".into(), Some("topic4 desc".into())),
        NewTopic::new("topic5".into(), Some("topic5 desc".into())),
        NewTopic::new("topic6".into(), Some("topic6 desc".into())),
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
#[tokio::test]
async fn patch_name_name_update(#[future] runtime: TestRuntime) {
    let runtime = runtime.await;
    let repo = runtime.repo;

    let created_topic = repo
        .create(NewTopic::new("topic1".into(), Some("topic1 desc".into())))
        .await
        .unwrap();

    let updated_topic = repo
        .patch(created_topic.id, PatchTopic::new(Some("topic2".into()), Field::Missing))
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
#[tokio::test]
async fn patch_name_desc_update(#[future] runtime: TestRuntime) {
    let runtime = runtime.await;
    let repo = runtime.repo;

    let created_topic = repo
        .create(NewTopic::new("topic1".into(), Some("topic1 desc".into())))
        .await
        .unwrap();

    let updated_topic = repo
        .patch(created_topic.id, PatchTopic::new(Some("topic2".into()), Field::Present(Some("topic2 desc".into()))))
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
#[tokio::test]
async fn patch_non_null_desc_update(#[future] runtime: TestRuntime) {
    let runtime = runtime.await;
    let repo = runtime.repo;

    let created_topic = repo
        .create(NewTopic::new("topic1".into(), Some("topic1 desc".into())))
        .await
        .unwrap();

    let updated_topic = repo
        .patch(created_topic.id, PatchTopic::new(None, Field::Present(Some("topic2 desc".into()))))
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
#[tokio::test]
async fn patch_null_desc_update(#[future] runtime: TestRuntime) {
    let runtime = runtime.await;
    let repo = runtime.repo;

    let created_topic = repo
        .create(NewTopic::new("topic1".into(), Some("topic1 desc".into())))
        .await
        .unwrap();

    let updated_topic = repo
        .patch(created_topic.id, PatchTopic::new(None, Field::Present(None)))
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
#[tokio::test]
async fn patch_no_updates_leaves_topic_alone(#[future] runtime: TestRuntime) {
    let runtime = runtime.await;
    let repo = runtime.repo;

    let created_topic = repo
        .create(NewTopic::new("topic1".into(), Some("topic1 desc".into())))
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
#[tokio::test]
async fn patch_no_created_topics_returns_none(#[future] runtime: TestRuntime) {
    let runtime = runtime.await;
    let repo = runtime.repo;

    let updated_topic = repo
        .patch(TopicId::new(), PatchTopic::new(None, Field::Missing))
        .await
        .unwrap();

    assert!(updated_topic.is_none());
}

#[rstest]
#[tokio::test]
async fn patch_topic_not_found_returns_none(#[future] runtime: TestRuntime) {
    let runtime = runtime.await;
    let repo = runtime.repo;

    let _ = repo
        .create(NewTopic::new("topic1".into(), Some("topic1 desc".into())))
        .await
        .unwrap();

    let updated_topic = repo
        .patch(TopicId::new(), PatchTopic::new(None, Field::Missing))
        .await
        .unwrap();

    assert!(updated_topic.is_none());
}

#[fixture]
async fn runtime(#[future] container: ContainerAsync<Postgres>) -> TestRuntime {
    let container = container.await;
    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();

    let repo = TopicRepo::init_with_pool_size(
        ConnectionDetails::Url(format!(
            "postgresql://testuser:testpass@{host}:{port}/topics"
        )),
        1,
    )
    .await
    .unwrap();

    TestRuntime {
        _container: container,
        repo,
    }
}

#[fixture]
async fn container() -> ContainerAsync<Postgres> {
    Postgres::default()
        .with_db_name("topics")
        .with_user("testuser")
        .with_password("testpass")
        .start()
        .await
        .unwrap()
}
