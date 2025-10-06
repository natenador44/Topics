use dotenv::dotenv;
use engine::Engine;
use repositories::postgres;
use tokio::runtime::Handle;
use topic_routes::AppProperties;
use tracing::{debug, error};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Clone)]
struct AppEngine {
    repo: postgres::TopicRepo,
}

impl AppEngine {
    fn new(repo: postgres::TopicRepo) -> Self {
        Self { repo }
    }
}

impl Engine for AppEngine {
    type Repo = postgres::TopicRepo;

    fn topics(&self) -> Self::Repo {
        self.repo.clone()
    }
}

#[tokio::main]
async fn main() {
    init_logging();

    if let Err(e) = dotenv() {
        debug!("failed to read .env file: {e}");
    }

    let repo = init_repo(create_connection_details()).await;
    if let Err(e) = topic_routes::run(AppEngine::new(repo), AppProperties { port: 3000 }).await {
        error!("failed to run topics app");
        error!("{e:?}");
    };
}

fn create_connection_details() -> postgres::ConnectionDetails {
    // let Ok(port) = std::env::var("POSTGRES_PORT")
    //     .map(|s| s.parse())
    //     .unwrap_or(Ok(5432))
    // else {
    //     error!("failed to parse POSTGRES_PORT");
    //     std::process::exit(1);
    // };

    // let Ok(user) = std::env::var("POSTGRES_USER") else {
    //     error!("failed to parse POSTGRES_USER");
    //     std::process::exit(1);
    // };

    // let Ok(password) = std::env::var("POSTGRES_PASSWORD") else {
    //     error!("failed to parse POSTGRES_PASSWORD");
    //     std::process::exit(1);
    // };

    let Ok(postgres_url) = std::env::var("POSTGRES_URL") else {
        error!("POSTGRES_URL environment variable is missing");
        std::process::exit(1);
    };

    postgres::ConnectionDetails::Url(postgres_url)
}

async fn init_repo(connection_details: postgres::ConnectionDetails) -> postgres::TopicRepo {
    match postgres::init(Handle::current(), connection_details).await {
        Ok(r) => r,
        Err(e) => {
            error!("failed to initialize repository: {e:?}");
            std::process::exit(2);
        }
    }
}

fn init_logging() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_env("TOPICS_LOG"))
        .init();
    // TODO file logging
}
