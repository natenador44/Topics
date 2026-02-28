use error_stack::IntoReport;
use routing::pagination::Pagination;
use topics_core::{
    TopicEngine,
    list_filter::TopicListCriteria,
    mock::{MockTopicRepository, matchers},
    model::{NewTopic, Topic},
    result::TopicRepoError,
};

type Id = usize;

use crate::service::{TopicCreation, TopicService};

#[derive(Clone)]
struct TestEngine(MockTopicRepository<Id>);
impl TopicEngine for TestEngine {
    type TopicId = Id;

    type Repo = MockTopicRepository<Id>;

    fn repo(&self) -> Self::Repo {
        self.0.clone()
    }
}

#[tokio::test]
async fn get_repo_returns_none_service_returns_none() {
    let mut repo = MockTopicRepository::default();
    repo.get_mock
        .when_called_with(matchers::anything())
        .then_return(|| Ok(None));
    let service = build_service(repo.clone());

    assert!(service.get(1).await.unwrap().is_none());

    repo.get_mock
        .verify_call_with(matchers::anything())
        .was_called_only_once();
}

#[tokio::test]
async fn get_repo_returns_topic_service_returns_it_unmodified() {
    const EXPECTED_ID: usize = 1;
    const EXPECTED_NAME: &str = "topic 1";
    const EXPECTED_DESC: &str = "topic 1 desc";
    let mut repo = MockTopicRepository::default();

    repo.get_mock
        .when_called_with(matchers::eq(EXPECTED_ID))
        .then_return(|| {
            Ok(Some(Topic::create(
                1usize,
                EXPECTED_NAME.into(),
                Some(EXPECTED_DESC.into()),
            )))
        });

    let service = build_service(repo.clone());

    let topic = service
        .get(EXPECTED_ID)
        .await
        .expect("get should succeed")
        .expect("topic should exist");

    assert_eq!(EXPECTED_ID, topic.id);
    assert_eq!(EXPECTED_NAME, &topic.name);
    assert_eq!(Some(EXPECTED_DESC), topic.description.as_deref());

    repo.get_mock
        .verify_call_with(matchers::eq(EXPECTED_ID))
        .was_called_only_once();
}

#[tokio::test]
async fn get_repo_returns_err_service_returns_err() {
    let mut repo = MockTopicRepository::default();
    repo.get_mock
        .when_called_with(matchers::anything())
        .then_return(|| Err(TopicRepoError::Get.into_report()));
    let service = build_service(repo.clone());

    assert!(service.get(1).await.is_err());

    repo.get_mock
        .verify_call_with(matchers::anything())
        .was_called_only_once();
}

const DEFAULT_PAGE_SIZE: u64 = 25;

#[tokio::test]
async fn list_no_data_in_repo_returns_empty_vec() {
    let mut repo = MockTopicRepository::default();
    repo.list_mock
        .when_called_with(matchers::anything())
        .then_return(|| Ok(vec![]));
    let service = build_service(repo.clone());

    assert_eq!(
        Vec::<Topic<Id>>::new(),
        service
            .list(default_list_criteria())
            .await
            .expect("list should succeed")
    );

    repo.list_mock
        .verify_call_with(matchers::anything())
        .was_called_only_once();

    repo.get_mock
        .verify_call_with(matchers::anything())
        .was_not_called();
}

#[tokio::test]
async fn list_returns_all_topics_returned_from_repo_if_le_page_size() {
    let mut repo = MockTopicRepository::default();
    repo.list_mock
        .when_called_with(matchers::anything())
        .then_return(|| Ok(create_topics(NUM_TOPICS)));

    const NUM_TOPICS: usize = 20;
    let service = build_service(repo.clone());

    assert_eq!(
        NUM_TOPICS,
        service
            .list(default_list_criteria())
            .await
            .expect("list should succeed")
            .len()
    );

    repo.list_mock
        .verify_call_with(matchers::anything())
        .was_called_only_once();
}

#[tokio::test]
async fn list_truncates_topics_returned_from_repo_if_gt_page_size() {
    const NUM_TOPICS: usize = DEFAULT_PAGE_SIZE as usize + 5;

    let mut repo = MockTopicRepository::default();
    repo.list_mock
        .when_called_with(matchers::eq(default_list_criteria()))
        .then_return(|| Ok(create_topics(NUM_TOPICS)));

    let service = build_service(repo.clone());

    assert_eq!(
        DEFAULT_PAGE_SIZE as usize,
        service
            .list(default_list_criteria())
            .await
            .expect("list should succeed")
            .len()
    );

    repo.list_mock
        .verify_call_with(matchers::eq(default_list_criteria()))
        .was_called_only_once();
}

#[tokio::test]
async fn list_returns_error_if_repo_returns_error() {
    let mut repo = MockTopicRepository::default();
    repo.list_mock
        .when_called_with(matchers::eq(default_list_criteria()))
        .then_return(|| Err(TopicRepoError::List.into_report()));

    let service = build_service(repo.clone());

    let _ = service
        .list(default_list_criteria())
        .await
        .expect_err("list should return error");

    repo.list_mock
        .verify_call_with(matchers::eq(default_list_criteria()))
        .was_called_only_once();
}

#[tokio::test]
async fn create_calls_repo_create_with_passed_in_name_and_desc_and_does_not_modify() {
    const EXPECTED_NAME: &str = "topic 1";
    const EXPECTED_DESC: &str = "topic 1 desc";

    let new_topic = NewTopic::new(EXPECTED_NAME, Some(EXPECTED_DESC));

    let mut repo = MockTopicRepository::default();
    repo.create_mock
        .when_called_with(matchers::eq(new_topic.clone()))
        .then_return(|| {
            Ok(Topic::create(
                1usize,
                EXPECTED_NAME.into(),
                Some(EXPECTED_DESC.into()),
            ))
        });

    let service = build_service(repo.clone());

    let created_topic = service
        .create(TopicCreation::new(
            EXPECTED_NAME.into(),
            Some(EXPECTED_DESC.into()),
        ))
        .await
        .expect("create topic succeeds");

    assert_eq!(EXPECTED_NAME, &created_topic.name);
    assert_eq!(Some(EXPECTED_DESC), created_topic.description.as_deref());

    repo.create_mock
        .verify_call_with(matchers::eq(new_topic))
        .was_called_only_once();
}

// TODO test that metrics are called, caching, etc.. too much work for now

fn build_service(repo: MockTopicRepository<usize>) -> TopicService<TestEngine> {
    TopicService::new(TestEngine(repo))
}

fn create_topics(amt: usize) -> Vec<Topic<Id>> {
    (0..amt)
        .map(|i| Topic::create(i, format!("topic {i}"), Some(format!("topic {i} desc"))))
        .collect()
}

fn default_list_criteria() -> TopicListCriteria {
    TopicListCriteria::new(Pagination::default(), DEFAULT_PAGE_SIZE)
}
