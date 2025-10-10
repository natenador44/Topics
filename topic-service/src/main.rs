use engine::app::{AppProperties, AppResult};
use mongodb::Client;
use topic_service::repository::TopicRepo;
use topic_service::service::TopicService;
use topic_service::state::TopicAppState;
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

    let db_connection_str = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "mongodb://admin:password@127.0.0.1:27017/?authSource=admin".to_string()
    });
    debug!("connection string: {}", db_connection_str);
    let client = Client::with_uri_str(db_connection_str).await.unwrap();

    let routes = topic_service::routes::build(TopicAppState::new(TopicService::new(
        TopicRepo::new(client),
    )));

    engine::app::run(routes, AppProperties { port: 3000 }).await
}
