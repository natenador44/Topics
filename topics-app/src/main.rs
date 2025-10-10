use axum::Router;
use engine::app::{AppError, AppProperties, AppResult};
use error_stack::ResultExt;
use topics_core::TopicRepository;
use topics_routes::service::TopicService;
use topics_routes::state::TopicAppState;
use tracing::{debug, error, info};
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

    let routes = topics_routes().await?;

    engine::app::run(routes, AppProperties { port: 3000 }).await
}

#[cfg(feature = "mongo-topic-repo")]
async fn topics_routes() -> AppResult<Router> {
    use repositories::mongodb::topics::{ConnectionDetails, TopicRepo};

    let db_connection_str = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "mongodb://admin:password@127.0.0.1:27017/?authSource=admin".to_string()
    });

    let repo = TopicRepo::init(ConnectionDetails::Url(db_connection_str))
        .await
        .change_context(AppError)?;
    Ok(topics_routes::routes::build(TopicAppState::new(
        TopicService::new(TopicEngine::new(repo)),
    )))
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
