use axum_test::TestServer;
use axum_test::http::StatusCode;
use mongodb::Client;
use rstest::{fixture, rstest};
use testcontainers_modules::mongo::Mongo;
use testcontainers_modules::testcontainers::ContainerAsync;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use topic_service::repository::TopicRepo;
use topic_service::service::TopicService;
use topic_service::state::TopicAppState;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

struct TestRuntime {
    _container: ContainerAsync<Mongo>,
    test_server: TestServer,
}

#[rstest]
#[tokio::test]
async fn no_data_no_content(#[future] runtime: TestRuntime) {
    let runtime = runtime.await;
    let server = runtime.test_server;

    server
        .get("/topics")
        .await
        .assert_status(StatusCode::NO_CONTENT);
}

#[fixture]
async fn runtime() -> TestRuntime {
    let container = Mongo::default().start().await.unwrap();
    let client = create_client(&container).await;
    let routes = topic_service::routes::build(TopicAppState::new(TopicService::new(
        TopicRepo::new(client),
    )));

    init_logging();

    TestRuntime {
        _container: container,
        test_server: TestServer::new(routes).unwrap(),
    }
}

fn init_logging() {
    tracing_subscriber::registry().with(fmt::layer()).init();
}

async fn create_client(container: &ContainerAsync<Mongo>) -> Client {
    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(27017).await.unwrap();
    Client::with_uri_str(format!("mongodb://{host}:{port}/"))
        .await
        .unwrap()
}
