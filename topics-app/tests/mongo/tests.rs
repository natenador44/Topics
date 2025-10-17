use axum_test::TestServer;
use axum_test::http::StatusCode;
use bson::oid::ObjectId;
use mongodb::Client;
use repositories::mongodb::topics::TopicRepo;
use rstest::{fixture, rstest};
use serde_json::Value;
use serde_json::json;
use testcontainers_modules::mongo::Mongo;
use testcontainers_modules::testcontainers::ContainerAsync;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use topics_core::TopicEngine;
use topics_core::TopicRepository;
use topics_core::model::Topic;
use topics_routes::service::TopicService;
use topics_routes::state::TopicAppState;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::Layer;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::mongo::migration::Migration;
use crate::mongo::migration::NewTopicCreated;

struct TestRuntime {
    _container: ContainerAsync<Mongo>,
    test_server: TestServer,
    client: Client,
}

macro_rules! int_test {
    ($name:ident => $f:expr) => {
        #[rstest]
        #[tokio::test]
        async fn $name(#[from(init)] _init: &(), #[future] runtime: TestRuntime) {
            let runtime = runtime.await;
            let server = runtime.test_server;
            let client = runtime.client;
            ($f)(server, client).await
        }
    };
}

int_test!(get_no_content_not_found => |server: TestServer, _: Client| async move {
    let id = ObjectId::new();
    server
        .get(&format!("/topics/{id}"))
        .await
        .assert_status_not_found();
});

int_test!(
    get_after_insert_finds_topic => |server: TestServer, client: Client| async move {
        let topic = NewTopicCreated::new("t1", Some("desc"));
        let ids = Migration::default()
            .single(topic.clone())
            .run(client)
            .await;

        let id = ids[0];

        let expected = Topic::new(id, topic.name, topic.description, topic.created, None);

        let actual: Topic<ObjectId> = server.get(&format!("/topics/{id}"))
            .await
            .json();

        assert_eq!(expected.id, actual.id);
        assert_eq!(expected.name, actual.name);
        assert_eq!(expected.description, actual.description);
        assert_eq!(expected.created, actual.created);
        assert_eq!(expected.updated, actual.updated);
    }
);

int_test!(
    get_non_existent_id_not_found => |server: TestServer, client: Client| async move {
        Migration::default()
            .fill(50)
            .run(client)
            .await;

        let unknown_id = ObjectId::new();

        server.get(&format!("/topics/{unknown_id}"))
            .await
            .assert_status_not_found();

    }
);

// ideally this would be an unprocessable entity, but can work on that later
int_test!(
    get_bad_id_not_found => |server: TestServer, _: Client| async move {
        let bad_id = "badid";

        server.get(&format!("/topic/{bad_id}"))
            .await
            .assert_status_not_found();
    }
);

int_test!(list_no_data_no_content => |server: TestServer, _: Client| async move {
    server
        .get("/topics")
        .await
        .assert_status(StatusCode::NO_CONTENT);
});

int_test!(
    some_data_ok =>
    |server: TestServer, client: Client| async move {
        Migration::default()
            .fill(5)
            .run(client).await;

        server
            .get("/topics")
            .await
            .assert_status_ok();
    }
);

int_test!(
    list_default_pagination_for_list_is_25 =>
    |server: TestServer, client: Client| async move {
        Migration::default()
            .fill(100)
            .run(client).await;

        let response = server
            .get("/topics")
            .await;

        let body: Vec<Value> = response.json();

        assert_eq!(25, body.len());
    }
);

int_test!(
    list_custom_page_size_is_used_if_specified =>
    |server: TestServer, client: Client| async move {
        Migration::default()
            .fill(100)
            .run(client).await;

        let response = server
            .get("/topics?page_size=5")
            .await;

        let body: Vec<Value> = response.json();

        assert_eq!(5, body.len());
    }
);

int_test!(
    list_default_page_is_1 =>
    |server: TestServer, client: Client| async move {
        let first_topic = NewTopicCreated::new("first", Some("first topic in the database"));
        Migration::default()
            .single(first_topic.clone())
            .fill(99)
            .run(client).await;

        let response = server
            .get("/topics")
            .await;

        let body: Vec<NewTopicCreated> = response.json();

        assert_eq!(25, body.len());
        assert_eq!(first_topic, body[0]);
    }
);

int_test!(
    list_custom_page_is_used_if_specified =>
    |server: TestServer, client: Client| async move {
        let first_topic = NewTopicCreated::new("second page topic", Some("second page topic desc"));
        Migration::default()
            .fill(25)
            .single(first_topic.clone())
            .fill(24)
            .run(client).await;

        let response = server
            .get("/topics?page=2")
            .await;

        let body: Vec<NewTopicCreated> = response.json();

        assert_eq!(25, body.len());
        assert_eq!(first_topic, body[0]);
    }
);

int_test!(
    list_custom_page_and_page_size_is_used_if_specified =>
    |server: TestServer, client: Client| async move {
        let first_topic = NewTopicCreated::new("second page topic", Some("second page topic desc"));
        Migration::default()
            .fill(25)
            .single(first_topic.clone())
            .fill(24)
            .run(client).await;

        let response = server
            .get("/topics?page=6&page_size=5")
            .await;

        let body: Vec<NewTopicCreated> = response.json();

        assert_eq!(5, body.len());
        assert_eq!(first_topic, body[0]);
    }
);

int_test!(
    create_null_name_unprocessable_entity => |server: TestServer, _: Client| async move {
        server.post("/topics")
            .json(&json!({
                "name": null,
                "description": "new topic"
            }))
            .await
            .assert_status_unprocessable_entity();
    }
);

int_test!(
    create_success_no_description => |server: TestServer, _: Client| async move {
        let res = server.post("/topics")
            .json(&json!({
                "name": "test topic",
                "description": null
            }))
            .await;

        res.assert_status(StatusCode::CREATED);
        let topic: Topic<ObjectId> = res.json();

        assert_eq!("test topic", &topic.name);
        assert!(topic.description.is_none());
    }
);

int_test!(
    create_success_description => |server: TestServer, _: Client| async move {
        let res = server.post("/topics")
            .json(&json!({
                "name": "test topic",
                "description": "test desc",
            }))
            .await;

        res.assert_status(StatusCode::CREATED);
        let topic: Topic<ObjectId> = res.json();

        assert_eq!("test topic", &topic.name);
        assert_eq!(Some("test desc"), topic.description.as_deref());
    }
);

int_test!(
    create_success_updated_is_none => |server: TestServer, _: Client| async move {
        let res = server.post("/topics")
            .json(&json!({
                "name": "test topic",
                "description": "test desc",
            }))
            .await;

        res.assert_status(StatusCode::CREATED);
        let topic: Topic<ObjectId> = res.json();

        assert!(topic.updated.is_none());
    }
);

#[fixture]
#[once]
fn init() -> () {
    init_logging();
}

#[fixture]
async fn runtime() -> TestRuntime {
    let container = Mongo::default().start().await.unwrap();
    let client = create_client(&container).await;
    let routes = topics_routes::routes::build(TopicAppState::new(TopicService::new(TestEngine {
        repo: TopicRepo::new(client.clone()),
    })));

    TestRuntime {
        _container: container,
        test_server: TestServer::new(routes).unwrap(),
        client,
    }
}

fn init_logging() {
    let log_level = std::env::var("TOPICS_TEST_LOG")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(LevelFilter::ERROR);

    tracing_subscriber::registry()
        .with(fmt::layer().with_filter(log_level))
        .init();
}

async fn create_client(container: &ContainerAsync<Mongo>) -> Client {
    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(27017).await.unwrap();
    Client::with_uri_str(format!("mongodb://{host}:{port}/"))
        .await
        .unwrap()
}

#[derive(Clone)]
struct TestEngine {
    repo: TopicRepo,
}

impl TopicEngine for TestEngine {
    type TopicId = <TopicRepo as TopicRepository>::TopicId;

    type Repo = TopicRepo;

    fn repo(&self) -> Self::Repo {
        self.repo.clone()
    }
}
