use crate::app::models::{SetId, TopicId};
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

// TODO test rejection of all endpoints if topic id or set id is not a UUID

#[tokio::test]
async fn create_set_success() {
    let topic_id = TopicId::new();
    let set_id = SetId::new();

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
    response.assert_json(&json!({
        "entities_url": format!("/api/v1/topics/{topic_id}/sets/{set_id}/entities"),
        "id": set_id,
        "topic_id": topic_id,
        "name": TEST_SET_NAME,
    }));
}

#[tokio::test]
async fn create_set_entities_are_optional() {
    let topic_id = TopicId::new();
    let set_id = SetId::new();

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
async fn create_set_name_json_type_is_non_null_string() {
    let topic_id = TopicId::new();

    let invalid_name_json = [
        Value::Null,
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
    )
    .await;

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
    topic_repo
        .expect_exists()
        .return_once(return_scenario::topic_exists(true));

    let response = run_post_endpoint(
        &format!("/api/v1/topics/{topic_id}/sets"),
        set_repo,
        topic_repo,
        json!({ "name": TEST_SET_NAME }),
    )
    .await;

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

#[tokio::test]
async fn get_set_success() {
    let topic_id = TopicId::new();
    let set_id = SetId::new();

    let mut set_repo = MockSetRepository::new();
    set_repo
        .expect_get()
        .with(predicate::eq(topic_id), predicate::eq(set_id))
        .return_once(return_scenario::get::found(topic_id, set_id, TEST_SET_NAME));

    let mut topic_repo = MockTopicRepository::new();
    topic_repo
        .expect_exists()
        .with(predicate::eq(topic_id))
        .return_once(return_scenario::topic_exists(true));

    let response = run_get_endpoint(
        &format!("/api/v1/topics/{topic_id}/sets/{set_id}"),
        set_repo,
        topic_repo,
    )
    .await;

    response.assert_status_ok();
    response.assert_json(&json!({
        "id": set_id,
        "topic_id": topic_id,
        "name": TEST_SET_NAME,
        "entities_url": format!("/api/v1/topics/{topic_id}/sets/{set_id}/entities"),
    }));
}

#[tokio::test]
async fn get_set_returns_not_found_if_topic_does_not_exist() {
    let topic_id = TopicId::new();
    let set_id = SetId::new();

    let set_repo = MockSetRepository::new();

    let mut topic_repo = MockTopicRepository::new();
    topic_repo
        .expect_exists()
        .with(predicate::eq(topic_id))
        .return_once(return_scenario::topic_exists(false));

    let response = run_get_endpoint(
        &format!("/api/v1/topics/{topic_id}/sets/{set_id}"),
        set_repo,
        topic_repo,
    )
    .await;

    response.assert_status_not_found();
}

#[tokio::test]
async fn get_set_returns_not_found_if_set_does_not_exist() {
    let topic_id = TopicId::new();
    let set_id = SetId::new();

    let mut set_repo = MockSetRepository::new();
    set_repo
        .expect_get()
        .with(predicate::eq(topic_id), predicate::eq(set_id))
        .return_once(return_scenario::get::not_found);

    let mut topic_repo = MockTopicRepository::new();
    topic_repo
        .expect_exists()
        .with(predicate::eq(topic_id))
        .return_once(return_scenario::topic_exists(true));

    let response = run_get_endpoint(
        &format!("/api/v1/topics/{topic_id}/sets/{set_id}"),
        set_repo,
        topic_repo,
    )
    .await;

    response.assert_status_not_found();
}

#[tokio::test]
async fn get_set_returns_internal_server_error_if_topic_repo_returns_error() {
    let topic_id = TopicId::new();
    let set_id = SetId::new();

    let set_repo = MockSetRepository::new();

    let mut topic_repo = MockTopicRepository::new();
    topic_repo
        .expect_exists()
        .with(predicate::eq(topic_id))
        .return_once(return_scenario::topic_error);

    let response = run_get_endpoint(
        &format!("/api/v1/topics/{topic_id}/sets/{set_id}"),
        set_repo,
        topic_repo,
    )
    .await;

    response.assert_status_internal_server_error();
}

#[tokio::test]
async fn get_set_returns_internal_server_error_if_set_repo_returns_error() {
    let topic_id = TopicId::new();
    let set_id = SetId::new();

    let mut set_repo = MockSetRepository::new();
    set_repo
        .expect_get()
        .return_once(return_scenario::get::error);

    let mut topic_repo = MockTopicRepository::new();
    topic_repo
        .expect_exists()
        .with(predicate::eq(topic_id))
        .return_once(return_scenario::topic_exists(true));

    let response = run_get_endpoint(
        &format!("/api/v1/topics/{topic_id}/sets/{set_id}"),
        set_repo,
        topic_repo,
    )
    .await;

    response.assert_status_internal_server_error();
}

#[tokio::test]
async fn delete_success() {
    let topic_id = TopicId::new();
    let set_id = SetId::new();

    let mut set_repo = MockSetRepository::new();
    set_repo
        .expect_delete()
        .return_once(return_scenario::delete::success);

    let mut topic_repo = MockTopicRepository::new();
    topic_repo
        .expect_exists()
        .return_once(return_scenario::topic_exists(true));

    let response = run_delete_endpoint(
        &format!("/api/v1/topics/{topic_id}/sets/{set_id}"),
        set_repo,
        topic_repo,
    )
    .await;

    response.assert_status(StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn delete_bad_request_if_topic_id_not_uuid() {
    let set_id = SetId::new();

    let set_repo = MockSetRepository::new();

    let topic_repo = MockTopicRepository::new();

    let response = run_delete_endpoint(
        &format!("/api/v1/topics/notauuid/sets/{set_id}"),
        set_repo,
        topic_repo,
    )
    .await;

    response.assert_status_bad_request();
}

#[tokio::test]
async fn delete_bad_request_if_set_id_not_uuid() {
    let topic_id = TopicId::new();

    let set_repo = MockSetRepository::new();

    let topic_repo = MockTopicRepository::new();

    let response = run_delete_endpoint(
        &format!("/api/v1/topics/{topic_id}/sets/notauuid"),
        set_repo,
        topic_repo,
    )
    .await;

    response.assert_status_bad_request();
}

#[tokio::test]
async fn delete_returns_error_if_repo_returns_error() {
    let topic_id = TopicId::new();
    let set_id = SetId::new();

    let mut set_repo = MockSetRepository::new();
    set_repo
        .expect_delete()
        .return_once(return_scenario::delete::error);

    let mut topic_repo = MockTopicRepository::new();
    topic_repo
        .expect_exists()
        .return_once(return_scenario::topic_exists(true));

    let response = run_delete_endpoint(
        &format!("/api/v1/topics/{topic_id}/sets/{set_id}"),
        set_repo,
        topic_repo,
    )
    .await;

    response.assert_status_internal_server_error();
}

#[tokio::test]
async fn delete_returns_not_found_if_topic_does_not_exist() {
    let topic_id = TopicId::new();
    let set_id = SetId::new();

    let set_repo = MockSetRepository::new();

    let mut topic_repo = MockTopicRepository::new();
    topic_repo
        .expect_exists()
        .with(predicate::eq(topic_id))
        .return_once(return_scenario::topic_exists(false));

    let response = run_delete_endpoint(
        &format!("/api/v1/topics/{topic_id}/sets/{set_id}"),
        set_repo,
        topic_repo,
    )
    .await;

    response.assert_status_not_found();
}

async fn run_get_endpoint(
    path: &str,
    set_repo: MockSetRepository,
    topic_repo: MockTopicRepository,
) -> TestResponse {
    let server = init_test_server(set_repo, topic_repo);

    server.get(path).await
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
    use crate::app::models::Set;
    use crate::app::models::{SetId, TopicId};
    use crate::app::repository::{SetRepoError, TopicRepoError};
    use crate::error::AppResult;
    use error_stack::IntoReport;
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

        pub fn success<'a, N: ToString + Send + Sync + 'static>(
            topic_id: TopicId,
            set_id: SetId,
            name: N,
        ) -> impl FnOnce(TopicId, String, Vec<Value>) -> SetMockReturn<'a, Set> {
            move |_, _, _| {
                async move {
                    Ok(Set {
                        id: set_id,
                        topic_id,
                        name: name.to_string(),
                    })
                }
                .boxed()
            }
        }

        pub fn error<'a>(_: TopicId, _: String, _: Vec<Value>) -> SetMockReturn<'a, Set> {
            async { Err(SetRepoError::Create.into_report()) }.boxed()
        }
    }

    pub mod get {
        use super::*;
        pub fn found(
            topic_id: TopicId,
            set_id: SetId,
            name: impl ToString,
        ) -> impl FnOnce(TopicId, SetId) -> SetMockReturn<'static, Option<Set>> {
            let name = name.to_string();
            move |_, _| {
                async move {
                    Ok(Some(Set {
                        id: set_id,
                        topic_id,
                        name,
                    }))
                }
                .boxed()
            }
        }

        pub fn not_found<'a>(_: TopicId, _: SetId) -> SetMockReturn<'a, Option<Set>> {
            async { Ok(None) }.boxed()
        }

        pub fn error<'a>(_: TopicId, _: SetId) -> SetMockReturn<'a, Option<Set>> {
            async { Err(SetRepoError::Get.into_report()) }.boxed()
        }
    }

    pub mod delete {
        use super::*;

        pub fn success<'a>(_: TopicId, _: SetId) -> SetMockReturn<'a, ()> {
            async { Ok(()) }.boxed()
        }

        pub fn error<'a>(_: TopicId, _: SetId) -> SetMockReturn<'a, ()> {
            async { Err(SetRepoError::Delete.into_report()) }.boxed()
        }
    }
}
