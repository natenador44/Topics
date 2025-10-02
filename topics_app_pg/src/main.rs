use tracing::error;
use engine::Engine;
use topic_routes::AppProperties;

#[derive(Debug, Clone)]
struct AppEngine;

impl Engine for AppEngine {
    type Repo = postgres_repository::TopicRepo;

    fn topics(&self) -> Self::Repo {
        todo!()
    }
}

#[tokio::main]
async fn main() {
    if let Err(e) = topic_routes::run(AppEngine, AppProperties { port: 3000 }).await {
        error!("failed to run topics app");
        error!("{e:?}");
    }
}
