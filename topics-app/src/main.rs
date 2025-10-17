use axum::Router;
use dotenv::dotenv;
use engine::app::{AppError, AppProperties, AppResult};
use error_stack::ResultExt;
use repositories::mongodb::topics::TopicRepo;
use topics_core::TopicRepository;
use topics_routes::service::TopicService;
use topics_routes::state::TopicAppState;
use tracing::{debug, error, info, instrument, warn};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() {
    match try_main().await {
        Ok(_) => info!("topic service shutting down"),
        Err(e) => {
            error!("topic service exited with error: {e:?}");
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

async fn try_main() -> AppResult<()> {
    init_logging();

    if let Err(e) = dotenv() {
        warn!("failed to load .env file: {e}");
    }

    let routes = build_routes().await?;

    engine::app::run(routes, AppProperties { port: 3001 }).await
}

async fn build_routes() -> AppResult<Router> {
    let engine = build_repo().await?;

    debug!("building routes..");
    Ok(topics_routes::routes::build(TopicAppState::new_with_metrics(TopicEngine::new(engine))))
    .inspect(|_| debug!("routes built"))
}

#[cfg(feature = "mongo-topic-repo")]
#[instrument]
async fn build_repo() -> AppResult<TopicRepo> {
    use repositories::mongodb::topics::{ConnectionDetails, TopicRepo};

    let db_connection_str = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "mongodb://admin:password@127.0.0.1:27017/?authSource=admin".to_string()
    });

    debug!("initializing mongodb repository");
    TopicRepo::init(ConnectionDetails::Url(db_connection_str))
        .await
        .change_context(AppError)
}

#[derive(Debug, Clone)]
struct TopicEngine<T> {
    repo: T,
}
impl<T> TopicEngine<T> {
    fn new(repo: T) -> Self {
        Self { repo }
    }
}

impl<T> topics_core::TopicEngine for TopicEngine<T>
where
    T: TopicRepository + Clone + Send + Sync + 'static,
{
    type TopicId = T::TopicId;
    type Repo = T;

    fn repo(&self) -> Self::Repo {
        self.repo.clone()
    }
}
