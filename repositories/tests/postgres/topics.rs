use chrono::Utc;
use engine::Pagination;
use repositories::postgres::topics::{ConnectionDetails, TopicId, TopicRepo};
use rstest::{fixture, rstest};
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{ContainerAsync, runners::AsyncRunner},
};
use topics_core::list_filter::TopicListCriteria;
use topics_core::model::{NewTopic, Topic};
use topics_core::{CreateManyFailReason, CreateManyTopicStatus, TopicRepository};

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

    let statuses = new_topics
        .iter()
        .map(|t| CreateManyTopicStatus::Pending {
            name: t.name.clone(),
            description: t.description.clone(),
        })
        .collect();

    repo.create_many(statuses).await.unwrap();

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
async fn create_many_all_non_pending_statuses_are_unchanged(#[future] runtime: TestRuntime) {
    let runtime = runtime.await;
    let repo = runtime.repo;

    let original = vec![
        CreateManyTopicStatus::Fail {
            topic_name: None,
            topic_description: None,
            reason: CreateManyFailReason::MissingName,
        },
        CreateManyTopicStatus::Success(Topic::new(
            TopicId::new(),
            "blah".into(),
            None,
            Utc::now(),
            None,
        )),
    ];

    let updated = repo.create_many(original.clone()).await.unwrap();

    assert_eq!(original, updated)
}

#[rstest]
#[tokio::test]
async fn create_many_single_pending_is_created(#[future] runtime: TestRuntime) {
    let runtime = runtime.await;
    let repo = runtime.repo;

    let original = vec![CreateManyTopicStatus::Pending {
        name: "blah".into(),
        description: Some("blah desc".into()),
    }];

    let updated = repo.create_many(original.clone()).await.unwrap();

    assert_eq!(1, updated.len());
    let status = &updated[0];

    let CreateManyTopicStatus::Success(topic) = status else {
        panic!("expected status to be Success");
    };

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

    let new_topics = [
        NewTopic::new("topic1".into(), Some("topic1 desc".into())),
        NewTopic::new("topic2".into(), Some("topic2 desc".into())),
        NewTopic::new("topic3".into(), Some("topic3 desc".into())),
        NewTopic::new("topic4".into(), Some("topic4 desc".into())),
        NewTopic::new("topic5".into(), Some("topic5 desc".into())),
        NewTopic::new("topic6".into(), Some("topic6 desc".into())),
    ];

    let original = new_topics
        .iter()
        .map(|t| CreateManyTopicStatus::Pending {
            name: t.name.clone(),
            description: t.description.clone(),
        })
        .collect::<Vec<_>>();

    let updated = repo.create_many(original).await.unwrap();

    assert_eq!(new_topics.len(), updated.len());

    for (i, topic_req) in new_topics.into_iter().enumerate() {
        let CreateManyTopicStatus::Success(created_topic) = &updated[i] else {
            panic!("expected status to be Success");
        };

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
async fn create_many_mixed_statuses(#[future] runtime: TestRuntime) {
    let runtime = runtime.await;
    let repo = runtime.repo;

    let original = vec![
        CreateManyTopicStatus::Fail {
            topic_name: Some("topic1".into()),
            topic_description: None,
            reason: CreateManyFailReason::MissingName,
        },
        CreateManyTopicStatus::Pending {
            name: "topic2".into(),
            description: Some("topic2 desc".into()),
        },
        CreateManyTopicStatus::Pending {
            name: "topic3".into(),
            description: Some("topic3 desc".into()),
        },
        CreateManyTopicStatus::Success(Topic::new(
            TopicId::new(),
            "topic4".into(),
            Some("topic4 desc".into()),
            Utc::now(),
            None,
        )),
        CreateManyTopicStatus::Fail {
            topic_name: Some("topic5".into()),
            topic_description: None,
            reason: CreateManyFailReason::MissingName,
        },
        CreateManyTopicStatus::Pending {
            name: "topic6".into(),
            description: Some("topic6 desc".into()),
        },
    ];

    let updated = repo.create_many(original.clone()).await.unwrap();

    assert_eq!(original[0], updated[0]);
    is_success_with_name_desc(&updated[1], "topic2", Some("topic2 desc"));
    is_success_with_name_desc(&updated[2], "topic3", Some("topic3 desc"));
    assert_eq!(original[3], updated[3]);
    assert_eq!(original[4], updated[4]);
    is_success_with_name_desc(&updated[5], "topic6", Some("topic6 desc"));
}

fn is_success_with_name_desc(
    status: &CreateManyTopicStatus<TopicId>,
    name: &str,
    description: Option<&str>,
) {
    match status {
        CreateManyTopicStatus::Success(topic) => {
            assert_eq!(name, &topic.name);
            assert_eq!(description, topic.description.as_deref());
        }
        other => panic!("expected status to be Success, got {:?}", other),
    }
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
