use crate::app;
use crate::app::models::TopicId;
use crate::app::repository::MockSetRepository;
use crate::app::services::SetService;
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
use serde::Serialize;
use serde_json::{Map, Number, Value, json};

const DEFAULT_NAME: &str = "topic1";
const DEFAULT_DESC: &str = "description1";

#[tokio::test]
#[cfg_attr(miri, ignore)]
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
#[cfg_attr(miri, ignore)]
async fn search_default_pagination() {
    let mut topic_repo = MockTopicRepository::new();
    topic_repo
        .expect_search()
        .with(
            predicate::eq(1),
            predicate::eq(app::services::DEFAULT_TOPIC_SEARCH_PAGE_SIZE),
            predicate::eq(vec![]),
        )
        .once()
        .return_once(return_scenario::search::empty);

    let _ = run_get_endpoint("/api/v1/topics", topic_repo).await;
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn invalid_page_param_returns_bad_request() {
    let topic_repo = MockTopicRepository::new();

    let response = run_get_endpoint("/api/v1/topics?page=hello", topic_repo).await;

    response.assert_status_bad_request();
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn invalid_page_size_param_returns_bad_request() {
    let topic_repo = MockTopicRepository::new();

    let response = run_get_endpoint("/api/v1/topics?page_size=hello", topic_repo).await;

    response.assert_status_bad_request();
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn unknown_params_are_ignored() {
    let mut topic_repo = MockTopicRepository::new();
    topic_repo
        .expect_search()
        .with(
            predicate::eq(1),
            predicate::eq(app::services::DEFAULT_TOPIC_SEARCH_PAGE_SIZE),
            predicate::eq(vec![]),
        )
        .once()
        .return_once(return_scenario::search::empty);

    let response = run_get_endpoint("/api/v1/topics?unknown=hello", topic_repo).await;

    response.assert_status_success();
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
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
#[cfg_attr(miri, ignore)]
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
#[cfg_attr(miri, ignore)]
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
#[cfg_attr(miri, ignore)]
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
#[cfg_attr(miri, ignore)]
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
#[cfg_attr(miri, ignore)]
async fn search_returns_ok_status_when_topics_are_found() {
    let mut topic_repo = MockTopicRepository::new();
    topic_repo
        .expect_search()
        .once()
        .return_once(return_scenario::search::non_empty(create_topic_list(10)));

    let response = run_get_endpoint("/api/v1/topics", topic_repo).await;

    response.assert_status_ok();
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn search_returns_array_of_topic_responses_when_ok() {
    let mut topic_repo = MockTopicRepository::new();
    let topics = create_topic_list(10);
    topic_repo
        .expect_search()
        .once()
        .return_once(return_scenario::search::non_empty(topics.clone()));

    let expected_responses = topics
        .into_iter()
        .map(|t| {
            json!({
                "id": t.id,
                "name": t.name,
                "description": t.description,
                "sets_url": format!("/api/v1/topics/{}/sets", t.id),
                "identifiers_url": format!("/api/v1/topics/{}/identifiers", t.id),
            })
        })
        .collect::<Vec<_>>();

    let response = run_get_endpoint("/api/v1/topics", topic_repo).await;

    response.assert_status_ok();
    response.assert_json(&expected_responses);
}

// the service itself currently can't return an error. may need to ensure that it's dependencies are
// all traits so they can all be mocked, and the service doesn't return any errors on its own
// unless it can be influenced by one of its dependencies.
#[tokio::test]
#[cfg_attr(miri, ignore)]
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
#[cfg_attr(miri, ignore)]
async fn get_returns_not_found_if_no_topics_exist() {
    let id = TopicId::new();
    let mut topic_repo = MockTopicRepository::new();
    topic_repo
        .expect_get()
        .once()
        .returning(return_scenario::get::not_found);

    let response = run_get_endpoint(&format!("/api/v1/topics/{id}"), topic_repo).await;

    response.assert_status_not_found()
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn get_success() {
    let request_id = TopicId::new();

    let existing_topic = Topic::new(
        request_id,
        DEFAULT_NAME.to_owned(),
        Some(DEFAULT_DESC.to_owned()),
    );

    let mut topic_repo = MockTopicRepository::new();
    topic_repo
        .expect_get()
        .with(predicate::eq(request_id))
        .once()
        .return_once(return_scenario::get::found(existing_topic.clone()));

    let response = run_get_endpoint(&format!("/api/v1/topics/{request_id}"), topic_repo).await;

    response.assert_status_ok();
    response.assert_json(&json!({
        "id": request_id,
        "name": DEFAULT_NAME,
        "description": DEFAULT_DESC,
        "sets_url": format!("/api/v1/topics/{request_id}/sets"),
        "identifiers_url": format!("/api/v1/topics/{request_id}/identifiers"),
    }));
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn get_returns_internal_server_error_if_repo_error_occurs() {
    let request_id = TopicId::new();

    let mut topic_repo = MockTopicRepository::new();
    topic_repo
        .expect_get()
        .with(predicate::eq(request_id))
        .once()
        .return_once(return_scenario::get::error());

    let response = run_get_endpoint(&format!("/api/v1/topics/{request_id}"), topic_repo).await;

    response.assert_status_internal_server_error();
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn get_returns_bad_request_if_id_is_not_uuid() {
    let request_id = "bad_id";

    let topic_repo = MockTopicRepository::new();

    let response = run_get_endpoint(&format!("/api/v1/topics/{request_id}"), topic_repo).await;

    response.assert_status_bad_request();
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn create_success() {
    let topic_id = TopicId::new();

    let new_topic = Topic::new(
        topic_id,
        DEFAULT_NAME.to_owned(),
        Some(DEFAULT_DESC.to_owned()),
    );

    let mut topic_repo = MockTopicRepository::new();
    topic_repo
        .expect_create()
        .with(
            predicate::eq(DEFAULT_NAME.to_string()),
            predicate::eq(Some(DEFAULT_DESC.to_string())),
        )
        .once()
        .return_once(return_scenario::create::success(new_topic.clone()));

    let response = run_post_endpoint(
        "/api/v1/topics",
        topic_repo,
        &json!({
            "name": DEFAULT_NAME,
            "description": DEFAULT_DESC,
        }),
    )
    .await;

    response.assert_status(StatusCode::CREATED);
    response.assert_json(&json!({
        "id": topic_id,
        "name": DEFAULT_NAME,
        "description": DEFAULT_DESC,
        "sets_url": format!("/api/v1/topics/{topic_id}/sets"),
        "identifiers_url": format!("/api/v1/topics/{topic_id}/identifiers"),
    }));
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn create_returns_internal_server_error_if_repo_returns_error() {
    let mut topic_repo = MockTopicRepository::new();
    topic_repo
        .expect_create()
        .with(
            predicate::eq(DEFAULT_NAME.to_string()),
            predicate::eq(Some(DEFAULT_DESC.to_string())),
        )
        .once()
        .return_once(return_scenario::create::error);

    let response = run_post_endpoint(
        "/api/v1/topics",
        topic_repo,
        &json!({
            "name": DEFAULT_NAME,
            "description": DEFAULT_DESC,
        }),
    )
    .await;

    response.assert_status_internal_server_error();
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn create_name_json_type_is_string() {
    let invalid_name_json = [
        Value::Array(vec![]),
        Value::Object(Map::new()),
        Value::Number(Number::from(2)),
        Value::Number(Number::from_f64(1.2).unwrap()),
        Value::Bool(true),
        Value::Bool(false),
    ];

    for json in invalid_name_json {
        let topic_repo = MockTopicRepository::new();

        let response = run_post_endpoint(
            "/api/v1/topics",
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
#[cfg_attr(miri, ignore)]
async fn create_description_json_type_is_string() {
    let invalid_name_json = [
        Value::Array(vec![]),
        Value::Object(Map::new()),
        Value::Number(Number::from(2)),
        Value::Number(Number::from_f64(1.2).unwrap()),
        Value::Bool(true),
        Value::Bool(false),
    ];

    for json in invalid_name_json {
        let topic_repo = MockTopicRepository::new();

        let response = run_post_endpoint(
            "/api/v1/topics",
            topic_repo,
            json!({
                "name": DEFAULT_NAME,
                "description": json,
            }),
        )
        .await;

        response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
    }
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn create_name_is_not_optional() {
    let topic_repo = MockTopicRepository::new();

    let response = run_post_endpoint(
        "/api/v1/topics",
        topic_repo,
        &json!({
            "description": DEFAULT_DESC,
        }),
    )
    .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn create_description_is_optional() {
    let mut topic_repo = MockTopicRepository::new();
    topic_repo
        .expect_create()
        .with(predicate::eq(DEFAULT_NAME.to_string()), predicate::eq(None))
        .once()
        .return_once(return_scenario::create::success(Topic::new_random_id(
            DEFAULT_NAME,
            DEFAULT_DESC,
        )));

    let response = run_post_endpoint(
        "/api/v1/topics",
        topic_repo,
        &json!({
            "name": DEFAULT_NAME,
        }),
    )
    .await;

    response.assert_status_success();
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn delete_returns_no_content_if_no_error() {
    let id = TopicId::new();
    let mut topic_repo = MockTopicRepository::new();
    topic_repo
        .expect_delete()
        .with(predicate::eq(id))
        .return_once(return_scenario::delete::success);

    let response = run_delete_endpoint(&format!("/api/v1/topics/{id}"), topic_repo).await;

    response.assert_status(StatusCode::NO_CONTENT)
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn delete_returns_not_found_if_topic_does_not_exist() {
    let id = TopicId::new();
    let mut topic_repo = MockTopicRepository::new();
    topic_repo
        .expect_delete()
        .with(predicate::eq(id))
        .return_once(return_scenario::delete::not_found);

    let response = run_delete_endpoint(&format!("/api/v1/topics/{id}"), topic_repo).await;

    response.assert_status_not_found();
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn delete_returns_bad_request_if_id_is_invalid() {
    let request_id = "bad_id";

    let topic_repo = MockTopicRepository::new();

    let response = run_delete_endpoint(&format!("/api/v1/topics/{request_id}"), topic_repo).await;

    response.assert_status_bad_request();
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn delete_returns_internal_server_error_if_repo_returns_error() {
    let id = TopicId::new();
    let mut topic_repo = MockTopicRepository::new();
    topic_repo
        .expect_delete()
        .with(predicate::eq(id))
        .return_once(return_scenario::delete::error);

    let response = run_delete_endpoint(&format!("/api/v1/topics/{id}"), topic_repo).await;

    response.assert_status_internal_server_error()
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn update_returns_ok_and_updated_topic_if_no_error() {
    let new_name = String::from("different name");
    let new_desc = String::from("different description");
    let id = TopicId::new();
    let updated_topic = Topic::new(id, new_name.clone(), Some(new_desc.clone()));

    let mut topic_repo = MockTopicRepository::new();

    topic_repo
        .expect_update()
        .return_once(return_scenario::update::success(updated_topic.clone()));

    let response = run_patch_endpoint(
        &format!("/api/v1/topics/{id}"),
        topic_repo,
        &json!({
            "name": new_name,
            "description": new_desc,
        }),
    )
    .await;

    response.assert_status_ok();
    response.assert_json(&updated_topic);
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn update_returns_bad_request_if_id_is_invalid() {
    let request_id = "bad_id";

    let topic_repo = MockTopicRepository::new();

    let response = run_patch_endpoint(
        &format!("/api/v1/topics/{request_id}"),
        topic_repo,
        json!({
            "name": "different name",
            "description": "different description",
        }),
    )
    .await;

    response.assert_status_bad_request();
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn update_returns_internal_server_error_if_repo_returns_error() {
    let request_id = TopicId::new();
    let mut topic_repo = MockTopicRepository::new();
    topic_repo
        .expect_update()
        .return_once(return_scenario::update::error);

    let response = run_patch_endpoint(
        &format!("/api/v1/topics/{request_id}"),
        topic_repo,
        json!({
            "name": "different name",
            "description": "different description",
        }),
    )
    .await;

    response.assert_status_internal_server_error()
}

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn update_returns_not_found_if_topic_id_does_not_exist() {
    let id = TopicId::new();

    let mut topic_repo = MockTopicRepository::new();

    topic_repo
        .expect_update()
        .return_once(return_scenario::update::not_found);

    let response = run_patch_endpoint(
        &format!("/api/v1/topics/{id}"),
        topic_repo,
        &json!({
            "name": "different name",
            "description": "different description",
        }),
    )
    .await;

    response.assert_status_not_found();
}

async fn run_get_endpoint(path: &str, topic_repo: MockTopicRepository) -> TestResponse {
    let server = init_test_server(topic_repo);

    server.get(path).await
}

async fn run_post_endpoint<T>(path: &str, topic_repo: MockTopicRepository, body: T) -> TestResponse
where
    T: Serialize,
{
    let server = init_test_server(topic_repo);

    server.post(path).json(&body).await
}

async fn run_patch_endpoint<T>(path: &str, topic_repo: MockTopicRepository, body: T) -> TestResponse
where
    T: Serialize,
{
    let server = init_test_server(topic_repo);

    server.patch(path).json(&body).await
}

async fn run_delete_endpoint(path: &str, topic_repo: MockTopicRepository) -> TestResponse {
    let server = init_test_server(topic_repo);

    server.delete(path).await
}

fn init_test_server(topic_repo: MockTopicRepository) -> TestServer {
    let repo = MockRepo::for_topics_test(topic_repo);
    let services = Service {
        topics: TopicService::new(repo.clone()),
        sets: SetService::new(repo),
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
    use crate::error::AppResult;
    use error_stack::IntoReport;
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
        ) -> BoxFuture<'a, AppResult<Vec<Topic>, TopicRepoError>> {
            async { Ok(vec![]) }.boxed()
        }

        pub fn non_empty<'a>(
            topics: Vec<Topic>,
        ) -> impl FnOnce(
            usize,
            usize,
            Vec<TopicFilter>,
        ) -> BoxFuture<'a, AppResult<Vec<Topic>, TopicRepoError>> {
            move |_, _, _| async move { Ok(topics) }.boxed()
        }

        pub fn error<'a>() -> impl FnOnce(
            usize,
            usize,
            Vec<TopicFilter>,
        )
            -> BoxFuture<'a, AppResult<Vec<Topic>, TopicRepoError>> {
            move |_, _, _| async move { Err(TopicRepoError::Search.into_report()) }.boxed()
        }
    }

    pub mod get {
        use super::*;
        use crate::app::models::TopicId;

        pub fn not_found<'a>(
            _: TopicId,
        ) -> BoxFuture<'a, AppResult<Option<Topic>, TopicRepoError>> {
            async { Ok(None) }.boxed()
        }

        pub fn found<'a>(
            topic: Topic,
        ) -> impl FnOnce(TopicId) -> BoxFuture<'a, AppResult<Option<Topic>, TopicRepoError>>
        {
            |_| async { Ok(Some(topic)) }.boxed()
        }

        pub fn error<'a>()
        -> impl FnOnce(TopicId) -> BoxFuture<'a, AppResult<Option<Topic>, TopicRepoError>> {
            move |_| async move { Err(TopicRepoError::Get.into_report()) }.boxed()
        }
    }

    pub mod create {
        use super::*;
        use crate::app::models::TopicId;

        pub fn success<'a>(
            topic: Topic,
        ) -> impl FnOnce(String, Option<String>) -> BoxFuture<'a, AppResult<Topic, TopicRepoError>>
        {
            move |_, _| async move { Ok(topic) }.boxed()
        }

        pub fn error<'a>(
            _: String,
            _: Option<String>,
        ) -> BoxFuture<'a, AppResult<Topic, TopicRepoError>> {
            async { Err(TopicRepoError::Create.into_report()) }.boxed()
        }
    }

    pub mod delete {
        use super::*;
        use crate::app::models::TopicId;
        use crate::app::services::ResourceOutcome;

        pub fn success<'a>(
            _: TopicId,
        ) -> BoxFuture<'a, AppResult<ResourceOutcome, TopicRepoError>> {
            async { Ok(ResourceOutcome::Found) }.boxed()
        }

        pub fn not_found<'a>(
            _: TopicId,
        ) -> BoxFuture<'a, AppResult<ResourceOutcome, TopicRepoError>> {
            async { Ok(ResourceOutcome::NotFound) }.boxed()
        }

        pub fn error<'a>(_: TopicId) -> BoxFuture<'a, AppResult<ResourceOutcome, TopicRepoError>> {
            async { Err(TopicRepoError::Delete.into_report()) }.boxed()
        }
    }

    pub mod update {
        use super::*;
        use crate::app::models::TopicId;

        pub fn success<'a>(
            topic: Topic,
        ) -> impl FnOnce(
            TopicId,
            Option<String>,
            Option<String>,
        ) -> BoxFuture<'a, AppResult<Option<Topic>, TopicRepoError>> {
            |_, _, _| async { Ok(Some(topic)) }.boxed()
        }

        pub fn not_found<'a>(
            _: TopicId,
            _: Option<String>,
            _: Option<String>,
        ) -> BoxFuture<'a, AppResult<Option<Topic>, TopicRepoError>> {
            async { Ok(None) }.boxed()
        }

        pub fn error<'a>(
            _: TopicId,
            _: Option<String>,
            _: Option<String>,
        ) -> BoxFuture<'a, AppResult<Option<Topic>, TopicRepoError>> {
            async { Err(TopicRepoError::Update.into_report()) }.boxed()
        }
    }
}
