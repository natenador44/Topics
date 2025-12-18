use apps::{AppError, AppProperties, AppResult};
use axum::Router;
use dotenv::dotenv;
use error_stack::ResultExt;
use error_stack::fmt::ColorMode;
use repositories::postgres::initializer::RepoCreator;
use topics_core::TopicRepository;
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
    error_stack::Report::set_color_mode(ColorMode::None);

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

    apps::run(routes, AppProperties { port: 3001 }).await
}

async fn build_routes() -> AppResult<Router> {
    let repo = build_repo().await?;

    debug!("building routes..");
    Ok(topics_routes::routes::build(
        TopicAppState::new_with_metrics(TopicEngine::new(repo))
            .await
            .change_context(AppError)?,
    ))
    .inspect(|_| debug!("routes built"))
}

// #[instrument]
// async fn build_repo() -> AppResult<TopicRepo> {
//     use repositories::mongodb::topics::{ConnectionDetails, TopicRepo};
//
//     let db_connection_str = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
//         "mongodb://admin:password@127.0.0.1:27017/?authSource=admin".to_string()
//     });
//
//     debug!("initializing mongodb repository");
//     TopicRepo::init(ConnectionDetails::Url(db_connection_str))
//         .await
//         .change_context(AppError)
// }

#[instrument]
async fn build_repo() -> AppResult<repositories::postgres::topics::TopicRepo> {
    use repositories::postgres::ConnectionDetails;

    let db_connection_str = std::env::var("DATABASE_URL")
        .change_context(AppError)
        .attach("DATABASE_URL is missing")?;

    debug!("initializing repository");
    RepoCreator::default()
        .with_topics()
        .create(ConnectionDetails::Url(db_connection_str), None)
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
