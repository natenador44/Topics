use mongodb::Client;
use tracing::{error, info};
use tracing_subscriber::{fmt, EnvFilter};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use engine::app::{AppProperties, AppResult};
use set_service::repository::SetRepo;
use set_service::service::SetService;
use set_service::state::SetAppState;

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
    let client = Client::with_uri_str(db_connection_str).await.unwrap();

    let routes = set_service::routes::build(SetAppState::new(SetService::new(SetRepo::new(client))));

    engine::app::run(routes, AppProperties { port: 3000 }).await
}
