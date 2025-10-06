use crate::app::routes;
use crate::app::services::{Service, SetService, TopicService};
use crate::app::state::AppState;
use crate::app::tests::{
    MockExistingSetRepo, MockExistingTopicRepo, MockSetRepo, MockTopicRepo, TestEngine,
};
use axum::http::StatusCode;
use axum_test::{TestResponse, TestServer};
use engine::models::{SetId, TopicId};
use mockall::predicate;
use serde::Serialize;
use serde_json::{Map, Number, Value, json};

const TEST_SET_NAME: &str = "test";

// TODO test rejection of all endpoints if topic id or set id is not a UUID

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn create_set_success() {
    let topic_id = TopicId::new();
    let set_id = SetId::new();

    let mut set_repo = MockSetRepo::new();
    set_repo
        .expect_create()
        .return_once(return_scenario::set::create::success(
            topic_id,
            set_id,
            TEST_SET_NAME,
        ));

    let mut existing_topic_repo = MockExistingTopicRepo::new();
    existing_topic_repo
        .expect_sets()
        .return_once(move || set_repo);

    let mut topic_repo = MockTopicRepo::new();
    topic_repo.expect_expect_existing().return_once(
        return_scenario::topic::expect_existing::found(existing_topic_repo),
    );

    let response = run_post_endpoint(
        &format!("/api/v1/topics/{topic_id}/sets"),
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
#[cfg_attr(miri, ignore)]
async fn create_set_entities_are_optional() {
    let topic_id = TopicId::new();
    let set_id = SetId::new();

    let mut set_repo = MockSetRepo::new();
    set_repo
        .expect_create()
        .return_once(return_scenario::set::create::success(
            topic_id,
            set_id,
            TEST_SET_NAME,
        ));

    let mut existing_topic_repo = MockExistingTopicRepo::new();
    existing_topic_repo
        .expect_sets()
        .return_once(move || set_repo);

    let mut topic_repo = MockTopicRepo::new();
    topic_repo.expect_expect_existing().return_once(
        return_scenario::topic::expect_existing::found(existing_topic_repo),
    );

    let response = run_post_endpoint(
        &format!("/api/v1/topics/{topic_id}/sets"),
        topic_repo,
        json!({
            "name": TEST_SET_NAME.to_string(),
        }),
    )
    .await;

    response.assert_status(StatusCode::CREATED);
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn create_set_name_is_not_optional() {
    let topic_id = TopicId::new();

    let topic_repo = MockTopicRepo::new();

    let response = run_post_endpoint(
        &format!("/api/v1/topics/{topic_id}/sets"),
        topic_repo,
        json!({
            "entities": Vec::<Value>::new(),
        }),
    )
    .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
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
        let topic_repo = MockTopicRepo::new();

        let response = run_post_endpoint(
            &format!("/api/v1/topics/{topic_id}/sets"),
            topic_repo,
            json!({
                    "name": json,
            }),
        )
        .await;

        response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    }
}
//
#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn create_set_returns_not_found_if_topic_does_not_exist() {
    let topic_id = TopicId::new();

    let mut topic_repo = MockTopicRepo::new();
    topic_repo
        .expect_expect_existing()
        .with(predicate::eq(topic_id))
        .return_once(return_scenario::topic::expect_existing::not_found);

    let response = run_post_endpoint(
        &format!("/api/v1/topics/{topic_id}/sets"),
        topic_repo,
        json!({ "name": TEST_SET_NAME }),
    )
    .await;

    response.assert_status_not_found();
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn create_set_returns_internal_server_error_if_topic_repo_returns_error() {
    let topic_id = TopicId::new();
    let mut topic_repo = MockTopicRepo::new();
    topic_repo
        .expect_expect_existing()
        .with(predicate::eq(topic_id))
        .return_once(return_scenario::topic::expect_existing::error);

    let response = run_post_endpoint(
        &format!("/api/v1/topics/{topic_id}/sets"),
        topic_repo,
        json!({ "name": TEST_SET_NAME }),
    )
    .await;

    response.assert_status_internal_server_error();
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn create_set_returns_internal_server_error_if_set_repo_returns_error() {
    let topic_id = TopicId::new();
    let mut set_repo = MockSetRepo::new();
    set_repo
        .expect_create()
        .return_once(return_scenario::set::create::error);

    let mut existing_topic_repo = MockExistingTopicRepo::new();
    existing_topic_repo
        .expect_sets()
        .return_once(move || set_repo);

    let mut topic_repo = MockTopicRepo::new();
    topic_repo.expect_expect_existing().return_once(
        return_scenario::topic::expect_existing::found(existing_topic_repo),
    );

    let response = run_post_endpoint(
        &format!("/api/v1/topics/{topic_id}/sets"),
        topic_repo,
        json!({ "name": TEST_SET_NAME }),
    )
    .await;

    response.assert_status_internal_server_error();
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
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
        let topic_repo = MockTopicRepo::new();

        let response = run_post_endpoint(
            &format!("/api/v1/topics/{topic_id}/sets"),
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
#[cfg_attr(miri, ignore)]
async fn get_set_success() {
    let topic_id = TopicId::new();
    let set_id = SetId::new();

    let mut set_repo = MockSetRepo::new();
    set_repo
        .expect_find()
        .with(predicate::eq(set_id))
        .return_once(return_scenario::set::get::found(
            topic_id,
            set_id,
            TEST_SET_NAME,
        ));

    let mut existing_topic_repo = MockExistingTopicRepo::new();
    existing_topic_repo
        .expect_sets()
        .return_once(move || set_repo);

    let mut topic_repo = MockTopicRepo::new();
    topic_repo
        .expect_expect_existing()
        .with(predicate::eq(topic_id))
        .return_once(return_scenario::topic::expect_existing::found(
            existing_topic_repo,
        ));

    let response = run_get_endpoint(
        &format!("/api/v1/topics/{topic_id}/sets/{set_id}"),
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
#[cfg_attr(miri, ignore)]
async fn get_set_returns_not_found_if_topic_does_not_exist() {
    let topic_id = TopicId::new();
    let set_id = SetId::new();

    let mut topic_repo = MockTopicRepo::new();
    topic_repo
        .expect_expect_existing()
        .with(predicate::eq(topic_id))
        .return_once(return_scenario::topic::expect_existing::not_found);

    let response = run_get_endpoint(
        &format!("/api/v1/topics/{topic_id}/sets/{set_id}"),
        topic_repo,
    )
    .await;

    response.assert_status_not_found();
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn get_set_returns_internal_server_error_if_topic_repo_returns_error() {
    let topic_id = TopicId::new();
    let set_id = SetId::new();

    let mut topic_repo = MockTopicRepo::new();
    topic_repo
        .expect_expect_existing()
        .with(predicate::eq(topic_id))
        .return_once(return_scenario::topic::expect_existing::error);

    let response = run_get_endpoint(
        &format!("/api/v1/topics/{topic_id}/sets/{set_id}"),
        topic_repo,
    )
    .await;

    response.assert_status_internal_server_error();
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn get_set_returns_internal_server_error_if_set_repo_returns_error() {
    let topic_id = TopicId::new();
    let set_id = SetId::new();

    let mut set_repo = MockSetRepo::new();
    set_repo
        .expect_find()
        .return_once(return_scenario::set::get::error);

    let mut existing_topic_repo = MockExistingTopicRepo::new();
    existing_topic_repo
        .expect_sets()
        .return_once(move || set_repo);

    let mut topic_repo = MockTopicRepo::new();
    topic_repo
        .expect_expect_existing()
        .with(predicate::eq(topic_id))
        .return_once(return_scenario::topic::expect_existing::found(
            existing_topic_repo,
        ));

    let response = run_get_endpoint(
        &format!("/api/v1/topics/{topic_id}/sets/{set_id}"),
        topic_repo,
    )
    .await;

    response.assert_status_internal_server_error();
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn delete_success() {
    let topic_id = TopicId::new();
    let set_id = SetId::new();

    let mut existing_set_repo = MockExistingSetRepo::new();

    existing_set_repo
        .expect_delete()
        .return_once(return_scenario::set::delete::success);

    let mut set_repo = MockSetRepo::new();
    set_repo
        .expect_expect_existing()
        .with(predicate::eq(set_id))
        .return_once(return_scenario::set::expect_existing::found(
            existing_set_repo,
        ));

    let mut existing_topic_repo = MockExistingTopicRepo::new();
    existing_topic_repo
        .expect_sets()
        .return_once(move || set_repo);

    let mut topic_repo = MockTopicRepo::new();
    topic_repo
        .expect_expect_existing()
        .with(predicate::eq(topic_id))
        .return_once(return_scenario::topic::expect_existing::found(
            existing_topic_repo,
        ));

    let response = run_delete_endpoint(
        &format!("/api/v1/topics/{topic_id}/sets/{set_id}"),
        topic_repo,
    )
    .await;

    response.assert_status(StatusCode::NO_CONTENT);
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn delete_bad_request_if_topic_id_not_uuid() {
    let set_id = SetId::new();

    let topic_repo = MockTopicRepo::new();

    let response = run_delete_endpoint(
        &format!("/api/v1/topics/notauuid/sets/{set_id}"),
        topic_repo,
    )
    .await;

    response.assert_status_bad_request();
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn delete_bad_request_if_set_id_not_uuid() {
    let topic_id = TopicId::new();

    let topic_repo = MockTopicRepo::new();

    let response = run_delete_endpoint(
        &format!("/api/v1/topics/{topic_id}/sets/notauuid"),
        topic_repo,
    )
    .await;

    response.assert_status_bad_request();
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn delete_returns_error_if_set_repo_returns_error() {
    let topic_id = TopicId::new();
    let set_id = SetId::new();

    let mut existing_set_repo = MockExistingSetRepo::new();

    existing_set_repo
        .expect_delete()
        .return_once(return_scenario::set::delete::error);

    let mut set_repo = MockSetRepo::new();
    set_repo
        .expect_expect_existing()
        .with(predicate::eq(set_id))
        .return_once(return_scenario::set::expect_existing::found(
            existing_set_repo,
        ));

    let mut existing_topic_repo = MockExistingTopicRepo::new();
    existing_topic_repo
        .expect_sets()
        .return_once(move || set_repo);

    let mut topic_repo = MockTopicRepo::new();
    topic_repo
        .expect_expect_existing()
        .with(predicate::eq(topic_id))
        .return_once(return_scenario::topic::expect_existing::found(
            existing_topic_repo,
        ));

    let response = run_delete_endpoint(
        &format!("/api/v1/topics/{topic_id}/sets/{set_id}"),
        topic_repo,
    )
    .await;

    response.assert_status_internal_server_error();
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn delete_returns_not_found_if_topic_does_not_exist() {
    let topic_id = TopicId::new();
    let set_id = SetId::new();

    let mut topic_repo = MockTopicRepo::new();
    topic_repo
        .expect_expect_existing()
        .with(predicate::eq(topic_id))
        .return_once(return_scenario::topic::expect_existing::not_found);

    let response = run_delete_endpoint(
        &format!("/api/v1/topics/{topic_id}/sets/{set_id}"),
        topic_repo,
    )
    .await;

    response.assert_status_not_found();
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn delete_returns_not_found_if_set_does_not_exist() {
    let topic_id = TopicId::new();
    let set_id = SetId::new();

    let mut set_repo = MockSetRepo::new();
    set_repo
        .expect_expect_existing()
        .with(predicate::eq(set_id))
        .return_once(return_scenario::set::expect_existing::not_found);

    let mut existing_topic_repo = MockExistingTopicRepo::new();
    existing_topic_repo
        .expect_sets()
        .return_once(move || set_repo);

    let mut topic_repo = MockTopicRepo::new();
    topic_repo
        .expect_expect_existing()
        .with(predicate::eq(topic_id))
        .return_once(return_scenario::topic::expect_existing::found(
            existing_topic_repo,
        ));

    let response = run_delete_endpoint(
        &format!("/api/v1/topics/{topic_id}/sets/{set_id}"),
        topic_repo,
    )
    .await;

    response.assert_status_not_found();
}

async fn run_get_endpoint(path: &str, topic_repo: MockTopicRepo) -> TestResponse {
    let server = init_test_server(topic_repo);

    server.get(path).await
}

async fn run_post_endpoint<T>(path: &str, topic_repo: MockTopicRepo, body: T) -> TestResponse
where
    T: Serialize,
{
    let server = init_test_server(topic_repo);

    server.post(path).json(&body).await
}

async fn run_put_endpoint<T>(path: &str, topic_repo: MockTopicRepo, body: T) -> TestResponse
where
    T: Serialize,
{
    let server = init_test_server(topic_repo);

    server.put(path).json(&body).await
}

async fn run_delete_endpoint(path: &str, topic_repo: MockTopicRepo) -> TestResponse {
    let server = init_test_server(topic_repo);

    server.delete(path).await
}

fn init_test_server(topic_repo: MockTopicRepo) -> TestServer {
    let repo = TestEngine::new(topic_repo);
    let services = Service {
        topics: TopicService::new(repo.clone()),
        sets: SetService::new(repo),
    };

    let app_state = AppState::new(services);

    let routes = routes::build(app_state);

    TestServer::new(routes).expect("creation of test server")
}

mod return_scenario {
    use crate::app::services::ResourceOutcome;
    use crate::error::AppResult;
    use engine::error::{RepoResult, SetRepoError, TopicRepoError};
    use engine::models::Set;
    use engine::models::SetId;
    use engine::models::TopicId;
    use error_stack::IntoReport;
    use futures::FutureExt;
    use futures::future::BoxFuture;
    use serde_json::Value;
    use chrono::Utc;

    type SetMockReturn<T> = RepoResult<T, SetRepoError>;
    type TopicMockReturn<'a, T> = BoxFuture<'a, AppResult<T, TopicRepoError>>;

    pub fn topic_exists<'a>(yes: bool) -> impl FnOnce(TopicId) -> TopicMockReturn<'a, bool> {
        move |_| async move { Ok(yes) }.boxed()
    }

    pub fn topic_error<'a>(_: TopicId) -> TopicMockReturn<'a, bool> {
        async move { Err(TopicRepoError::Exists.into_report()) }.boxed()
    }

    pub mod topic {
        use super::*;
        pub mod expect_existing {
            use super::*;
            use crate::app::tests::MockExistingTopicRepo;

            pub fn found(
                mock_existing_topic_repo: MockExistingTopicRepo,
            ) -> impl FnOnce(TopicId) -> RepoResult<Option<MockExistingTopicRepo>, TopicRepoError>
            {
                move |_| Ok(Some(mock_existing_topic_repo))
            }

            pub fn not_found(
                _: TopicId,
            ) -> RepoResult<Option<MockExistingTopicRepo>, TopicRepoError> {
                Ok(None)
            }

            pub fn error(_: TopicId) -> RepoResult<Option<MockExistingTopicRepo>, TopicRepoError> {
                Err(TopicRepoError::Get.into_report())
            }
        }
    }

    pub mod set {
        use super::*;

        pub mod expect_existing {
            use super::*;
            use crate::app::tests::MockExistingSetRepo;

            pub fn found(
                mock_existing_topic_repo: MockExistingSetRepo,
            ) -> impl FnOnce(SetId) -> RepoResult<Option<MockExistingSetRepo>, SetRepoError>
            {
                move |_| Ok(Some(mock_existing_topic_repo))
            }

            pub fn not_found(_: SetId) -> RepoResult<Option<MockExistingSetRepo>, SetRepoError> {
                Ok(None)
            }

            pub fn error(_: TopicId) -> RepoResult<Option<MockExistingSetRepo>, SetRepoError> {
                Err(SetRepoError::Get.into_report())
            }
        }

        pub mod create {
            use super::*;

            pub fn success<N: ToString + Send + Sync + 'static>(
                topic_id: TopicId,
                set_id: SetId,
                name: N,
            ) -> impl FnOnce(String, Option<String>, Vec<Value>) -> SetMockReturn<Set> {
                move |_, _, _| {
                    Ok(Set {
                        id: set_id,
                        topic_id,
                        name: name.to_string(),
                        description: None,
                        created: Utc::now(),
                        updated: None,
                    })
                }
            }

            pub fn error<'a>(_: String, _: Option<String>, _: Vec<Value>) -> SetMockReturn<Set> {
                Err(SetRepoError::Create.into_report())
            }
        }

        pub mod get {
            use super::*;
            pub fn found(
                topic_id: TopicId,
                set_id: SetId,
                name: impl ToString,
            ) -> impl FnOnce(SetId) -> SetMockReturn<Option<Set>> {
                let name = name.to_string();
                move |_| {
                    Ok(Some(Set {
                        id: set_id,
                        topic_id,
                        name,
                        description: None,
                        created: Utc::now(),
                        updated: None,            
                    }))
                }
            }

            pub fn error<'a>(_: SetId) -> SetMockReturn<Option<Set>> {
                Err(SetRepoError::Get.into_report())
            }
        }

        pub mod delete {
            use super::*;

            pub fn success<'a>() -> SetMockReturn<()> {
                Ok(())
            }

            pub fn not_found<'a>(_: TopicId, _: SetId) -> SetMockReturn<ResourceOutcome> {
                Ok(ResourceOutcome::NotFound)
            }

            pub fn error<'a>() -> SetMockReturn<()> {
                Err(SetRepoError::Delete.into_report())
            }
        }
    }
}
