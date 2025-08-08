use axum::Router;

const TOPIC_ROOT_PATH: &str = "/topics";

pub fn routes() -> Router {
    Router::new().nest(TOPIC_ROOT_PATH, v1::routes())
}

mod v1 {
    const VERSION: &str = "/v1";
    use axum::{
        Router,
        extract::{Path, Query},
        response::IntoResponse,
        routing::get,
    };
    use serde::Deserialize;
    use topic_core::{TopicId, pagination::Pagination};
    use tracing::info;

    pub fn routes() -> Router {
        Router::new().nest(
            VERSION,
            Router::new()
                .route("/", get(search_topics))
                .route("/{topic_id}", get(get_topic)),
        )
    }

    #[derive(Debug, Deserialize)]
    struct SearchCriteria {
        pagination: Option<Pagination>,
        name: Option<String>,
        description: Option<String>,
    }

    async fn search_topics(Query(search_criteria): Query<SearchCriteria>) -> impl IntoResponse {
        info!("{search_criteria:?}");
    }

    async fn get_topic(Path(topic_id): Path<TopicId>) -> impl IntoResponse {}
}
