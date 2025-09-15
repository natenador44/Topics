use crate::app::{
    repository::MockTopicRepository,
    routes,
    services::{Service, TopicService},
    state::AppState,
    tests::MockRepo,
};
use axum::http::StatusCode;
use axum_test::TestServer;
use error_stack::Result;
use futures::FutureExt;

#[tokio::test]
async fn no_created_topics_returns_no_content() {
    let mut topic_repo = MockTopicRepository::new();
    topic_repo
        .expect_search()
        .once()
        .return_once(|_page, _page_size, _filters| async { Ok(vec![]) }.boxed());

    let server = init_test_server(topic_repo);

    let response = server.get("/api/v1/topics").await;

    response.assert_status(StatusCode::NO_CONTENT);
}

fn init_test_server(topic_repo: MockTopicRepository) -> TestServer {
    let services = Service {
        topics: TopicService::new(MockRepo::for_topics_test(topic_repo)),
    };

    let app_state = AppState::new(services);

    let routes = routes::build(app_state);

    TestServer::new(routes).expect("creation of test server")
}
