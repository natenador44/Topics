use axum::{
    Router,
    routing::{delete, get, post, put},
};
use const_format::formatcp;
use serde::Deserialize;

mod entities;
mod topics;

const VERSION: &str = "/v1";
const TOPICS_PATH: &str = "/topics";
const ENTITY_PATH: &str = "/entities";

pub fn routes() -> Router {
    let merged = topic_routes().merge(entity_routes());
    Router::new().nest(VERSION, merged)
}

fn topic_routes() -> Router {
    Router::new()
        .route(TOPICS_PATH, get(topics::search))
        .route(TOPICS_PATH, post(topics::create))
        .route(formatcp!("{}/{{topic_id}}", TOPICS_PATH), get(topics::get))
        .route(
            formatcp!("{}/{{topic_id}}", TOPICS_PATH),
            put(topics::update),
        )
        .route(
            formatcp!("{}/{{topic_id}}", TOPICS_PATH),
            delete(topics::delete),
        )
}

fn entity_routes() -> Router {
    Router::new()
}
