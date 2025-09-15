use crate::app::{
    models::Topic,
    repository::{MockTopicRepository, TopicFilter},
    routes,
    services::{Service, TopicService},
    state::AppState,
    tests::MockRepo,
};
use axum::http::StatusCode;
use axum_test::{TestResponse, TestServer};
use mockall::predicate;
use uuid::Uuid;
use crate::app;

const DEFAULT_NAME: &str = "topic1";
const DEFAULT_DESC: &str = "description1";

#[tokio::test]
async fn search_no_created_topics_returns_no_content() {
    let mut topic_repo = MockTopicRepository::new();
    topic_repo
        .expect_search()
        .once()
        .return_once(return_scenario::search::empty);

    let response = run_get_endpoint("/api/v1/topics", topic_repo).await;

    response.assert_status(StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn search_default_pagination() {
    let mut topic_repo = MockTopicRepository::new();
    topic_repo
        .expect_search()
        .with(predicate::eq(1), predicate::eq(app::services::DEFAULT_TOPIC_SEARCH_PAGE_SIZE), predicate::eq(vec![]))
        .once()
        .return_once(return_scenario::search::empty);

    let _ = run_get_endpoint("/api/v1/topics", topic_repo).await;
}

#[tokio::test]
async fn invalid_page_param_returns_bad_request() {
    let topic_repo = MockTopicRepository::new();

    let response = run_get_endpoint("/api/v1/topics?page=hello", topic_repo).await;

    response.assert_status_bad_request();
}

#[tokio::test]
async fn invalid_page_size_param_returns_bad_request() {
    let topic_repo = MockTopicRepository::new();

    let response = run_get_endpoint("/api/v1/topics?page_size=hello", topic_repo).await;

    response.assert_status_bad_request();
}

#[tokio::test]
async fn unknown_params_are_ignored() {
    let mut topic_repo = MockTopicRepository::new();
    topic_repo
        .expect_search()
        .with(predicate::eq(1), predicate::eq(app::services::DEFAULT_TOPIC_SEARCH_PAGE_SIZE), predicate::eq(vec![]))
        .once()
        .return_once(return_scenario::search::empty);

    let response = run_get_endpoint("/api/v1/topics?unknown=hello", topic_repo).await;

    response.assert_status_success();
}


#[tokio::test]
async fn search_page_param_is_taken_from_uri_query() {
    let page = 15;

    let mut topic_repo = MockTopicRepository::new();
    topic_repo
        .expect_search()
        .with(
            predicate::eq(page),
            predicate::eq(25),
            predicate::eq(vec![]),
        )
        .once()
        .return_once(return_scenario::search::empty);

    let _ = run_get_endpoint(&format!("/api/v1/topics?page={page}"), topic_repo).await;
}

#[tokio::test]
async fn search_page_size_param_is_taken_from_uri_query() {
    let page_size = 150;

    let mut topic_repo = MockTopicRepository::new();
    topic_repo
        .expect_search()
        .with(
            predicate::eq(1),
            predicate::eq(page_size),
            predicate::eq(vec![]),
        )
        .once()
        .return_once(return_scenario::search::empty);

    let _ = run_get_endpoint(&format!("/api/v1/topics?page_size={page_size}"), topic_repo).await;
}

#[tokio::test]
async fn search_name_param_results_in_topic_filter() {
    let name = String::from("topic1");

    let mut topic_repo = MockTopicRepository::new();
    topic_repo
        .expect_search()
        .with(
            predicate::eq(1),
            predicate::eq(25),
            predicate::eq(vec![TopicFilter::Name(name.clone())]),
        )
        .once()
        .return_once(return_scenario::search::empty);

    let _ = run_get_endpoint(&format!("/api/v1/topics?name={name}"), topic_repo).await;
}

#[tokio::test]
async fn search_description_param_results_in_topic_filter() {
    let description = String::from("desc1");

    let mut topic_repo = MockTopicRepository::new();
    topic_repo
        .expect_search()
        .with(
            predicate::eq(1),
            predicate::eq(25),
            predicate::eq(vec![TopicFilter::Description(description.clone())]),
        )
        .once()
        .return_once(return_scenario::search::empty);

    let _ = run_get_endpoint(
        &format!("/api/v1/topics?description={description}"),
        topic_repo,
    )
    .await;
}

#[tokio::test]
async fn search_name_and_description_param_results_in_topic_filters() {
    let name = String::from("name1");
    let description = String::from("desc1");

    let mut topic_repo = MockTopicRepository::new();
    topic_repo
        .expect_search()
        .with(
            predicate::eq(1),
            predicate::eq(25),
            predicate::eq(vec![
                // order matters here - maybe too strict?
                TopicFilter::Name(name.clone()),
                TopicFilter::Description(description.clone()),
            ]),
        )
        .once()
        .return_once(return_scenario::search::empty);

    let _ = run_get_endpoint(
        &format!("/api/v1/topics?name={name}&description={description}"),
        topic_repo,
    )
    .await;
}

#[tokio::test]
async fn search_returns_ok_status_when_topics_are_found() {
    let mut topic_repo = MockTopicRepository::new();
    topic_repo
        .expect_search()
        .once()
        .return_once(return_scenario::search::non_empty(create_topic_list(10)));

    let response = run_get_endpoint("/api/v1/topics", topic_repo).await;

    response.assert_status_ok();
}

// the service itself currently can't return an error. may need to ensure that it's dependencies are
// all traits so they can all be mocked, and the service doesn't return any errors on its own
// unless it can be influenced by one of its dependencies.
#[tokio::test]
async fn search_returns_internal_server_error_if_repo_returns_error() {
    let mut topic_repo = MockTopicRepository::new();
    topic_repo
        .expect_search()
        .once()
        .return_once(return_scenario::search::error());

    let response = run_get_endpoint("/api/v1/topics", topic_repo).await;

    response.assert_status_internal_server_error();
}

#[tokio::test]
async fn get_returns_not_found_if_no_topics_exist() {
    let id = Uuid::now_v7();
    let mut topic_repo = MockTopicRepository::new();
    topic_repo.expect_get().once().returning(return_scenario::get::not_found);

    let response = run_get_endpoint(&format!("/api/v1/topics/{id}"), topic_repo).await;

    response.assert_status_not_found()
}

#[tokio::test]
async fn get_returns_json_topic_and_ok_status_if_topic_found() {
    let existing_topic = Topic::new_random_id(DEFAULT_NAME, DEFAULT_DESC);

    let request_id = Uuid::now_v7();

    let mut topic_repo = MockTopicRepository::new();
    topic_repo.expect_get().with(predicate::eq(request_id)).once().return_once(return_scenario::get::found(existing_topic.clone()));

    let response = run_get_endpoint(&format!("/api/v1/topics/{request_id}"), topic_repo).await;

    response.assert_status_ok();
    response.assert_json(&existing_topic);
}

#[tokio::test]
async fn get_returns_internal_server_error_if_repo_error_occurs() {
    let request_id = Uuid::now_v7();

    let mut topic_repo = MockTopicRepository::new();
    topic_repo.expect_get().with(predicate::eq(request_id)).once().return_once(return_scenario::get::error());

    let response = run_get_endpoint(&format!("/api/v1/topics/{request_id}"), topic_repo).await;

    response.assert_status_internal_server_error();
}

#[tokio::test]
async fn get_returns_bad_request_if_id_is_not_uuid() {
    let request_id = "bad_id";

    let topic_repo = MockTopicRepository::new();

    let response = run_get_endpoint(&format!("/api/v1/topics/{request_id}"), topic_repo).await;

    response.assert_status_bad_request();
}
async fn run_get_endpoint(path: &str, topic_repo: MockTopicRepository) -> TestResponse {
    let server = init_test_server(topic_repo);

    server.get(path).await
}

fn init_test_server(topic_repo: MockTopicRepository) -> TestServer {
    let services = Service {
        topics: TopicService::new(MockRepo::for_topics_test(topic_repo)),
    };

    let app_state = AppState::new(services);

    let routes = routes::build(app_state);

    TestServer::new(routes).expect("creation of test server")
}

fn create_topic_list(amount: usize) -> Vec<Topic> {
    (0..amount)
        .map(|_| Topic::new_random_id(DEFAULT_NAME, DEFAULT_DESC))
        .collect()
}

mod return_scenario {
    use error_stack::{Result, report};
    use futures::{FutureExt, future::BoxFuture};

    use crate::app::{
        models::Topic,
        repository::{TopicFilter, TopicRepoError},
    };

    pub mod search {
        use super::*;
        pub fn empty<'a>(
            _: usize,
            _: usize,
            _: Vec<TopicFilter>,
        ) -> BoxFuture<'a, Result<Vec<Topic>, TopicRepoError>> {
            async { Ok(vec![]) }.boxed()
        }

        pub fn non_empty<'a>(
            topics: Vec<Topic>,
        ) -> impl FnOnce(
            usize,
            usize,
            Vec<TopicFilter>,
        ) -> BoxFuture<'a, Result<Vec<Topic>, TopicRepoError>> {
            move |_, _, _| async move { Ok(topics) }.boxed()
        }

        pub fn error<'a>() -> impl FnOnce(
            usize,
            usize,
            Vec<TopicFilter>,
        ) -> BoxFuture<'a, Result<Vec<Topic>, TopicRepoError>> {
            move |_, _, _| async move { Err(report!(TopicRepoError::Search)) }.boxed()
        }
    }

    pub mod get {
        use uuid::Uuid;
        use crate::app::models::TopicId;
        use super::*;

        pub fn not_found<'a>(_: Uuid) -> BoxFuture<'a, Result<Option<Topic>, TopicRepoError>> {
            async { Ok(None) }.boxed()
        }

        pub fn found<'a>(topic: Topic) -> impl FnOnce(TopicId) -> BoxFuture<'a, Result<Option<Topic>, TopicRepoError>> {
            |_| {
                async { Ok(Some(topic)) }.boxed()
            }
        }

        pub fn error<'a>() -> impl FnOnce(
            TopicId
        ) -> BoxFuture<'a, Result<Option<Topic>, TopicRepoError>> {
            move |_| async move { Err(report!(TopicRepoError::Get)) }.boxed()
        }
    }
}
