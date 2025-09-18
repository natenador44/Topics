use axum_test::expect_json::__private::serde_trampoline::ser::Serialize;
use axum_test::{TestResponse, TestServer};
use crate::app::repository::MockSetRepository;
use crate::app::routes;
use crate::app::services::{Service, SetService, TopicService};
use crate::app::state::AppState;
use crate::app::tests::MockRepo;

#[tokio::test]
async fn search_entities_in_set_returns_not_found_if_topic_id_does_not_exist() {

}

async fn run_post_endpoint<T>(path: &str, topic_repo: MockSetRepository, body: T) -> TestResponse
where
    T: Serialize,
{
    let server = init_test_server(topic_repo);

    server.post(path).json(&body).await
}

async fn run_put_endpoint<T>(path: &str, topic_repo: MockSetRepository, body: T) -> TestResponse
where
    T: Serialize,
{
    let server = init_test_server(topic_repo);

    server.put(path).json(&body).await
}

async fn run_delete_endpoint(path: &str, topic_repo: MockSetRepository) -> TestResponse {
    let server = init_test_server(topic_repo);

    server.delete(path).await
}

fn init_test_server(set_repo: MockSetRepository) -> TestServer {
    let repo = MockRepo::for_sets_test(set_repo);
    let services = Service {
        topics: TopicService::new(repo.clone()),
        sets: SetService::new(repo),
    };

    let app_state = AppState::new(services);

    let routes = routes::build(app_state);

    TestServer::new(routes).expect("creation of test server")
}
