use std::path::PathBuf;
use std::sync::Arc;

use axum_test::TestServer;
use repositories::postgres::ConnectionDetails;
use repositories::postgres::initializer::RepoCreator;
use repositories::postgres::topics::TopicId;
use repositories::postgres::topics::TopicRepo;
use reqwest::StatusCode;
use routing::AuthState;
use routing::OAuthConfig;
use rstest::{fixture, rstest};
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use testcontainers::compose::DockerCompose;
use testcontainers::core::RawContainer;
use testcontainers::core::WaitFor;
use testcontainers::core::wait::LogWaitStrategy;
use tokio::sync::OnceCell;
use topics_core::TopicEngine;
use topics_core::TopicRepository;
use topics_core::model::Topic;
use topics_routes::state::TopicAppState;
use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::Layer;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

const TEST_POSTGRES_USER: &str = "test";
const TEST_POSTGRES_PASSWORD: &str = "test";
const TEST_KEYCLOAK_USER: &str = "admin";
const TEST_KEYCLOACK_PASSWORD: &str = "admin";

#[rstest]
#[tokio::test]
async fn list_topics(#[future(awt)] context: Context) {
    let server = &context.runtime.server;

    let response = server
        .get("/topics")
        .authorization_bearer(&context.tokens.read_access)
        .await;

    assert_eq!(
        StatusCode::NO_CONTENT,
        response.status_code(),
        "GET /topics with read role is allowed",
    );

    let response = server.get("/topics").await;

    assert_eq!(
        StatusCode::UNAUTHORIZED,
        response.status_code(),
        "GET /topics without authorization is unauthorized"
    );

    let response = server
        .get("/topics")
        .authorization_bearer(&context.tokens.write_access)
        .await;

    assert_eq!(
        StatusCode::FORBIDDEN,
        response.status_code(),
        "GET /topics with write role is forbidden",
    );

    let response = server
        .post("/topics")
        .authorization_bearer(&context.tokens.write_access)
        .json(&json! ({
            "name": "test topic",
            "description": "test topic description",
        }))
        .await;
    assert_eq!(
        StatusCode::CREATED,
        response.status_code(),
        "POST /topics to test GET /topics with single topic created",
    );

    let response = server
        .get("/topics")
        .authorization_bearer(&context.tokens.read_access)
        .await;

    assert_eq!(
        StatusCode::OK,
        response.status_code(),
        "GET /topics with read role and single topic created is OK",
    );

    let topics: Vec<Topic<TopicId>> = response.json();
    assert_eq!(
        1,
        topics.len(),
        "GET /topics returns a single result after only creating one"
    );
    let topic = &topics[0];
    assert_eq!(
        "test topic", &topic.name,
        "GET /topics single created topic name matches"
    );
    assert_eq!(
        Some("test topic description"),
        topic.description.as_deref(),
        "GET /topics singel created topic description matches"
    );
}

struct TestRuntime {
    _containers: DockerCompose,
    server: TestServer,
}

struct Context {
    runtime: TestRuntime,
    tokens: Tokens,
}

#[derive(Debug, Clone)]
struct Tokens {
    read_access: Arc<str>,
    write_access: Arc<str>,
}

static LOGGING: OnceCell<()> = OnceCell::const_new();

async fn generate_tokens(auth_server: &RawContainer) -> Tokens {
    info!("generating tokens for tests to use..");
    let token_generator = token_generator(auth_server).await;
    let tokens = Tokens {
        read_access: token_generator.gen_read_token().await.into(),
        write_access: token_generator.gen_write_token().await.into(),
    };
    info!("tokens generated");
    tokens
}

// async fn start_auth_server() -> ContainerAsync<GenericImage> {
//     let keycloak_config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
//         .join("tests/resources/keycloak/realm-export.json");

//     info!("keycloak config path: {}", keycloak_config_path.display());

//     let image = GenericImage::new("test_keycloak", "latest")
//         .with_wait_for(WaitFor::Log(LogWaitStrategy::stdout(
//             "Listening on: http://0.0.0.0:8080",
//         )))
//         .with_exposed_port(ContainerPort::Tcp(8080))
//         .with_name("quay.io/keycloak/keycloak")
//         .with_env_var("KC_BOOTSTRAP_ADMIN_USERNAME", "admin")
//         .with_env_var("KC_BOOTSTRAP_ADMIN_PASSWORD", "admin")
//         .with_env_var("KC_HOSTNAME_STRICT", "false")
//         .with_env_var("KC_HOSTNAME_STRICT_HTTPS", "false")
//         .with_env_var("KC_HTTP_ENABLED", "true")
//         .with_cmd(["start-dev", "--import-realm"])
//         .with_mount(Mount::bind_mount(
//             keycloak_config_path.display().to_string(),
//             "/opt/keycloak/data/import/realm-export.json",
//         ));

//     info!("starting auth server - this may take a few seconds..");
//     let auth_server = image.start().await.expect("auth server should start");
//     info!("auth server started");
//     auth_server
// }

struct TokenGenerator {
    token_url: String,
}
#[derive(Deserialize)]
struct TokenGenRes {
    access_token: String,
}

#[derive(Serialize)]
struct TokenGenReq {
    grant_type: &'static str,
    client_id: &'static str,
    username: &'static str,
    password: &'static str,
}

const READ_USER_TOKEN_REQ: TokenGenReq = TokenGenReq {
    grant_type: "password",
    client_id: "token_generator",
    username: "reader@example.com",
    password: "password123",
};

const WRITE_USER_TOKEN_REQ: TokenGenReq = TokenGenReq {
    grant_type: "password",
    client_id: "token_generator",
    username: "writer@example.com",
    password: "password123",
};

impl TokenGenerator {
    async fn gen_read_token(&self) -> String {
        self.gen_token_for_user(&READ_USER_TOKEN_REQ).await
    }
    async fn gen_write_token(&self) -> String {
        self.gen_token_for_user(&WRITE_USER_TOKEN_REQ).await
    }

    async fn gen_token_for_user(&self, req: &TokenGenReq) -> String {
        let res = reqwest::Client::new()
            .post(&self.token_url)
            .form(req)
            .send()
            .await
            .expect("reading 'read' user token should succeed");

        let res: TokenGenRes = res
            .json()
            .await
            .expect("parsing 'read' user token response as json should succeed");

        res.access_token
    }
}

async fn token_generator(auth_server: &RawContainer) -> TokenGenerator {
    let auth_server_host = auth_server
        .get_host()
        .await
        .expect("auth server host exists");
    let auth_server_port = auth_server
        .get_host_port_ipv4(8080)
        .await
        .expect("port mapping for 8080");

    let token_url = format!(
        "http://{auth_server_host}:{auth_server_port}/realms/topics-realm/protocol/openid-connect/token"
    );
    TokenGenerator { token_url }
}

// I think the plan for different types of repos is feature flags. Each of these will be guarded by a feature flag, and all named `context`
// It should run all the same tests, with a different repo (like postgres vs mognodb)
#[fixture]
pub async fn context() -> Context {
    info!("creating context for test..");
    let _ = LOGGING.get_or_init(|| async { init_logging() }).await;

    let test_containers = test_containers().await;

    let auth_server = test_containers
        .service("auth_server")
        .expect("auth server should have started");

    let tokens = generate_tokens(&auth_server).await;
    let oauth_config = oauth(&auth_server).await;
    let repo = repo(&test_containers).await;

    info!("creating app state for test..");
    let app_state = TopicAppState::new_without_metrics(TestEngine { repo })
        .await
        .expect("creation of topic app state");

    info!("building routes for test..");
    let routes = topics_routes::routes::build(
        app_state,
        AuthState::create_with(oauth_config)
            .await
            .expect("auth state created"),
    );

    let ctx = Context {
        runtime: TestRuntime {
            _containers: test_containers,
            server: TestServer::new(routes).expect("test server created"),
        },
        tokens,
    };

    info!("context created!");

    ctx
}

async fn repo(test_containers: &DockerCompose) -> TopicRepo {
    let postgres = test_containers
        .service("database")
        .expect("postgres container exists");

    let host = postgres
        .get_host()
        .await
        .expect("postgres container host found");
    let port = postgres
        .get_host_port_ipv4(5432)
        .await
        .expect("postgres container port found");

    let db_connection_str =
        format!("postgresql://{TEST_POSTGRES_USER}:{TEST_POSTGRES_PASSWORD}@{host}:{port}/topics");

    info!("initializing repository with url {db_connection_str}");
    RepoCreator::default()
        .with_topics()
        .create(ConnectionDetails::Url(db_connection_str), Some(1))
        .await
        .expect("topic repo created")
}

async fn test_containers() -> DockerCompose {
    let dc_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/resources/docker-compose.yml");

    info!("starting test containers, this might take a bit..");

    let mut compose = DockerCompose::with_local_client(&[dc_path])
        .with_env("POSTGRES_USER", TEST_POSTGRES_USER)
        .with_env("POSTGRES_PASSWORD", TEST_POSTGRES_PASSWORD)
        .with_env("KEYCLOAK_USER", TEST_KEYCLOAK_USER)
        .with_env("KEYCLOAK_PASSWORD", TEST_KEYCLOACK_PASSWORD)
        .with_wait_for_service(
            "auth_server",
            WaitFor::Log(LogWaitStrategy::stdout("Listening on: http://0.0.0.0:8080")),
        )
        .with_wait_for_service(
            "database",
            WaitFor::Log(LogWaitStrategy::stderr(
                "database system is ready to accept connections",
            )),
        );

    compose.up().await.expect("test containers should start");
    info!("test containers started!");
    info!("services: [{}]", compose.services().join(","));

    compose
}

async fn oauth(auth_server: &RawContainer) -> OAuthConfig {
    #[derive(Deserialize)]
    struct OpenIdConfig {
        issuer: String,
        jwks_uri: String,
    }

    info!("building oauth config..");

    let auth_server_host = auth_server
        .get_host()
        .await
        .expect("auth server host exists");
    let auth_server_port = auth_server
        .get_host_port_ipv4(8080)
        .await
        .expect("port mapping for 8080");

    let open_id_config: OpenIdConfig = reqwest::get(format!(
        "http://{auth_server_host}:{auth_server_port}/realms/topics-realm/.well-known/openid-configuration"
    ))
    .await
    .expect("auth server should be available")
    .json()
    .await
    .expect("response body from auth server should be parseable as OpenIdConfig");

    let config = routing::OAuthConfig {
        jwks_url: open_id_config.jwks_uri,
        issuer_url: open_id_config.issuer,
        roles_claims_path: "roles".into(),
        audience: "topics-api".into(),
    };

    info!("oauth config created: {config:?}");
    config
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
