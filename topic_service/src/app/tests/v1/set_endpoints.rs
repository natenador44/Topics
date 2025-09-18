use crate::app::models::{TopicId, TopicSetId};
use crate::app::repository::{MockSetRepository, MockTopicRepository};
use crate::app::routes;
use crate::app::services::{Service, SetService, TopicService};
use crate::app::state::AppState;
use crate::app::tests::MockRepo;
use axum::http::StatusCode;
use axum_test::expect_json::__private::serde_trampoline::ser::Serialize;
use axum_test::{TestResponse, TestServer};
use mockall::predicate;
use serde_json::{Map, Number, Value, json};

const TEST_SET_NAME: &str = "test";

#[tokio::test]
async fn create_set_returns_created_if_successful() {
    let topic_id = TopicId::new();
    let set_id = TopicSetId::new();

    let mut set_repo = MockSetRepository::new();
    set_repo
        .expect_create()
        .return_once(return_scenario::create::success(
            topic_id,
            set_id,
            TEST_SET_NAME,
        ));

    let mut topic_repo = MockTopicRepository::new();
    topic_repo
        .expect_exists()
        .return_once(return_scenario::topic_exists(true));

    let response = run_post_endpoint(
        &format!("/api/v1/topics/{topic_id}/sets"),
        set_repo,
        topic_repo,
        json!({
            "name": TEST_SET_NAME.to_string(),
            "entities": Vec::<Value>::new(),
        }),
    )
    .await;

    response.assert_status(StatusCode::CREATED);
}

#[tokio::test]
async fn create_set_entities_are_optional() {
    let topic_id = TopicId::new();
    let set_id = TopicSetId::new();

    let mut set_repo = MockSetRepository::new();
    set_repo
        .expect_create()
        .return_once(return_scenario::create::success(
            topic_id,
            set_id,
            TEST_SET_NAME,
        ));

    let mut topic_repo = MockTopicRepository::new();
    topic_repo
        .expect_exists()
        .return_once(return_scenario::topic_exists(true));

    let response = run_post_endpoint(
        &format!("/api/v1/topics/{topic_id}/sets"),
        set_repo,
        topic_repo,
        json!({
            "name": TEST_SET_NAME.to_string(),
        }),
    )
    .await;

    response.assert_status(StatusCode::CREATED);
}

#[tokio::test]
async fn create_set_name_is_not_optional() {
    let topic_id = TopicId::new();

    let set_repo = MockSetRepository::new();

    let topic_repo = MockTopicRepository::new();

    let response = run_post_endpoint(
        &format!("/api/v1/topics/{topic_id}/sets"),
        set_repo,
        topic_repo,
        json!({
            "entities": Vec::<Value>::new(),
        }),
    )
    .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn create_set_name_json_type_is_string() {
    let topic_id = TopicId::new();

    let invalid_name_json = [
        Value::Array(vec![]),
        Value::Object(Map::new()),
        Value::Number(Number::from(2)),
        Value::Number(Number::from_f64(1.2).unwrap()),
        Value::Bool(true),
        Value::Bool(false),
    ];

    for json in invalid_name_json {
        let set_repo = MockSetRepository::new();
        let topic_repo = MockTopicRepository::new();

        let response = run_post_endpoint(
            &format!("/api/v1/topics/{topic_id}/sets"),
            set_repo,
            topic_repo,
            json!({
                    "name": json,
            }),
        )
        .await;

        response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    }
}

#[tokio::test]
async fn create_set_returns_not_found_if_topic_does_not_exist() {
    let topic_id = TopicId::new();
    let set_repo = MockSetRepository::new();
    let mut topic_repo = MockTopicRepository::new();
    topic_repo
        .expect_exists()
        .with(predicate::eq(topic_id))
        .return_once(return_scenario::topic_exists(false));

    let response = run_post_endpoint(
        &format!("/api/v1/topics/{topic_id}/sets"),
        set_repo,
        topic_repo,
        json!({ "name": TEST_SET_NAME }),
    )
    .await;

    response.assert_status_not_found();
}

#[tokio::test]
async fn create_set_returns_internal_server_error_if_topic_repo_returns_error() {
    let topic_id = TopicId::new();
    let set_repo = MockSetRepository::new();
    let mut topic_repo = MockTopicRepository::new();
    topic_repo
        .expect_exists()
        .with(predicate::eq(topic_id))
        .return_once(return_scenario::topic_error);

    let response = run_post_endpoint(
        &format!("/api/v1/topics/{topic_id}/sets"),
        set_repo,
        topic_repo,
        json!({ "name": TEST_SET_NAME }),
    ).await;

    response.assert_status_internal_server_error();
}

#[tokio::test]
async fn create_set_returns_internal_server_error_if_set_repo_returns_error() {
    let topic_id = TopicId::new();
    let mut set_repo = MockSetRepository::new();
    set_repo
        .expect_create()
        .return_once(return_scenario::create::error);
    let mut topic_repo = MockTopicRepository::new();
    topic_repo.expect_exists().return_once(return_scenario::topic_exists(true));

    let response = run_post_endpoint(
        &format!("/api/v1/topics/{topic_id}/sets"),
        set_repo,
        topic_repo,
        json!({ "name": TEST_SET_NAME }),
    ).await;

    response.assert_status_internal_server_error();
}

#[tokio::test]
async fn create_set_entities_json_type_is_array() {
    let topic_id = TopicId::new();

    let invalid_name_json = [
        Value::String("hello".into()),
        Value::Object(Map::new()),
        Value::Number(Number::from(2)),
        Value::Number(Number::from_f64(1.2).unwrap()),
        Value::Bool(true),
        Value::Bool(false),
    ];

    for json in invalid_name_json {
        let set_repo = MockSetRepository::new();
        let topic_repo = MockTopicRepository::new();

        let response = run_post_endpoint(
            &format!("/api/v1/topics/{topic_id}/sets"),
            set_repo,
            topic_repo,
            json!({
                    "name": TEST_SET_NAME,
                    "entities": json,
            }),
        )
        .await;

        response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    }
}

async fn run_post_endpoint<T>(
    path: &str,
    set_repo: MockSetRepository,
    topic_repo: MockTopicRepository,
    body: T,
) -> TestResponse
where
    T: Serialize,
{
    let server = init_test_server(set_repo, topic_repo);

    server.post(path).json(&body).await
}

async fn run_put_endpoint<T>(
    path: &str,
    set_repo: MockSetRepository,
    topic_repo: MockTopicRepository,
    body: T,
) -> TestResponse
where
    T: Serialize,
{
    let server = init_test_server(set_repo, topic_repo);

    server.put(path).json(&body).await
}

async fn run_delete_endpoint(
    path: &str,
    set_repo: MockSetRepository,
    topic_repo: MockTopicRepository,
) -> TestResponse {
    let server = init_test_server(set_repo, topic_repo);

    server.delete(path).await
}

fn init_test_server(set_repo: MockSetRepository, topic_repo: MockTopicRepository) -> TestServer {
    let repo = MockRepo::for_sets_test(set_repo, topic_repo);
    let services = Service {
        topics: TopicService::new(repo.clone()),
        sets: SetService::new(repo),
    };

    let app_state = AppState::new(services);

    let routes = routes::build(app_state);

    TestServer::new(routes).expect("creation of test server")
}

mod return_scenario {
    use error_stack::IntoReport;
    use crate::app::models::TopicSet;
    use crate::app::models::{TopicId, TopicSetId};
    use crate::app::repository::{SetRepoError, TopicRepoError};
    use crate::error::AppResult;
    use futures::FutureExt;
    use futures::future::BoxFuture;
    use serde_json::Value;

    type SetMockReturn<'a, T> = BoxFuture<'a, AppResult<T, SetRepoError>>;
    type TopicMockReturn<'a, T> = BoxFuture<'a, AppResult<T, TopicRepoError>>;

    pub fn topic_exists<'a>(yes: bool) -> impl FnOnce(TopicId) -> TopicMockReturn<'a, bool> {
        move |_| async move { Ok(yes) }.boxed()
    }

    pub fn topic_error<'a>(_: TopicId) -> TopicMockReturn<'a, bool> {
        async move { Err(TopicRepoError::Exists.into_report()) }.boxed()
    }

    pub mod create {
        use super::*;
        use futures::future::BoxFuture;
        use tracing_subscriber::filter::FilterExt;

        pub fn success<'a, N: ToString + Send + Sync + 'static>(
            topic_id: TopicId,
            set_id: TopicSetId,
            name: N,
        ) -> impl FnOnce(TopicId, String, Vec<Value>) -> SetMockReturn<'a, TopicSet> {
            move |_, _, _| {
                async move {
                    Ok(TopicSet {
                        id: set_id,
                        topic_id,
                        name: name.to_string(),
                    })
                }
                .boxed()
            }
        }

        pub fn error<'a>(_: TopicId, _: String, _: Vec<Value>) -> SetMockReturn<'a, TopicSet> {
            async { Err(SetRepoError::Create.into_report()) }.boxed()
        }
    }
}
