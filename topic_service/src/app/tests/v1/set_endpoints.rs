use axum::http::StatusCode;
use crate::app::models::{TopicId, TopicSetId};
use crate::app::repository::{MockSetRepository, MockTopicRepository};
use crate::app::routes;
use crate::app::services::{Service, SetService, TopicService};
use crate::app::state::AppState;
use crate::app::tests::MockRepo;
use axum_test::expect_json::__private::serde_trampoline::ser::Serialize;
use axum_test::{TestResponse, TestServer};
use serde_json::{json, Value};

const TEST_SET_NAME: &str = "test";

#[tokio::test]
async fn create_set_returns_created_if_successful() {
    let topic_id = TopicId::new();
    let set_id = TopicSetId::new();

    let mut set_repo = MockSetRepository::new();
    set_repo
        .expect_create()
        .return_once(return_scenario::create::success(topic_id, set_id, TEST_SET_NAME));

    let mut topic_repo = MockTopicRepository::new();
    topic_repo.expect_exists()
        .return_once(return_scenario::topic_exists(true));

    let response = run_post_endpoint(&format!("/api/v1/topics/{topic_id}/sets"), set_repo, topic_repo, json!({
        "name": TEST_SET_NAME.to_string(),
        "entities": Vec::<Value>::new(),
    })).await;

    response.assert_status(StatusCode::CREATED);
}

#[tokio::test]
async fn create_set_returns_created_even_if_entities_are_not_sent_in_request() {
    let topic_id = TopicId::new();
    let set_id = TopicSetId::new();

    let mut set_repo = MockSetRepository::new();
    set_repo
        .expect_create()
        .return_once(return_scenario::create::success(topic_id, set_id, TEST_SET_NAME));

    let mut topic_repo = MockTopicRepository::new();
    topic_repo.expect_exists()
        .return_once(return_scenario::topic_exists(true));

    let response = run_post_endpoint(&format!("/api/v1/topics/{topic_id}/sets"), set_repo, topic_repo, json!({
        "name": TEST_SET_NAME.to_string(),
    })).await;

    response.assert_status(StatusCode::CREATED);
}

async fn run_post_endpoint<T>(path: &str, set_repo: MockSetRepository, topic_repo: MockTopicRepository, body: T) -> TestResponse
where
    T: Serialize,
{
    let server = init_test_server(set_repo, topic_repo);

    server.post(path).json(&body).await
}

async fn run_put_endpoint<T>(path: &str, set_repo: MockSetRepository, topic_repo: MockTopicRepository, body: T) -> TestResponse
where
    T: Serialize,
{
    let server = init_test_server(set_repo, topic_repo);

    server.put(path).json(&body).await
}

async fn run_delete_endpoint(path: &str, set_repo: MockSetRepository,topic_repo: MockTopicRepository,) -> TestResponse {
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
    use futures::future::BoxFuture;
    use crate::app::models::TopicSet;
    use crate::app::models::{TopicId, TopicSetId};
    use crate::app::repository::{SetRepoError, TopicRepoError};
    use crate::error::AppResult;
    use futures::FutureExt;
    use serde_json::Value;

    type SetMockReturn<'a, T> = BoxFuture<'a, AppResult<T, SetRepoError>>;
    type TopicMockReturn<'a, T> = BoxFuture<'a, AppResult<T, TopicRepoError>>;

    pub fn topic_exists<'a>(yes: bool) -> impl FnOnce(TopicId) -> TopicMockReturn<'a, bool> {
        move |_| async move { Ok(yes) }.boxed()
    }

    pub mod create {
        use super::*;
        use futures::future::BoxFuture;
        pub fn success<'a, N: ToString + Send + Sync + 'static>(
            topic_id: TopicId,
            set_id: TopicSetId,
            name: N,
        ) -> impl FnOnce(TopicId, String, Vec<Value>) -> SetMockReturn<'a, TopicSet>
        {
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
    }
}
