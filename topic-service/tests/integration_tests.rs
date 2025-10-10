use axum_test::http::StatusCode;
use axum_test::TestServer;
use mongodb::Client;
use rstest::{fixture, rstest};
use testcontainers_modules::mongo::Mongo;
use testcontainers_modules::testcontainers::ContainerAsync;
use testcontainers_modules::testcontainers::runners::{AsyncRunner};
use topic_service::repository::TopicRepo;
use topic_service::service::TopicService;
use topic_service::state::TopicAppState;

#[rstest]
#[tokio::test]
async fn no_data_no_content(#[future] server: TestServer) {
    let server = server.await;

    server.get("/topics")
        .await
        .assert_status(StatusCode::NO_CONTENT);
}

#[fixture]
async fn server(#[future] client: Client) -> TestServer {
    let client = client.await;
    let routes = topic_service::routes::build(TopicAppState::new(TopicService::new(TopicRepo::new(client))));

    TestServer::new(routes).unwrap()
}

#[fixture]
async fn client(#[future] container: ContainerAsync<Mongo>) -> Client {
    let container = container.await;
    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(27017).await.unwrap();
    Client::with_uri_str(format!("mongodb://{host}:{port}/")).await.unwrap()
}

#[fixture]
async fn container() -> ContainerAsync<Mongo> {
    Mongo::default()
        .start()
        .await
        .unwrap()
}
