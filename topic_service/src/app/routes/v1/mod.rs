use axum::{
    Router,
    routing::{MethodRouter, Route, delete, get, post, put},
};
use const_format::{concatcp, formatcp};

mod entities;
mod topics;

const VERSION: &str = "/v1";
const TOPICS_PATH: &str = "topics";
const ENTITIES_PATH: &str = "entities";

pub fn routes() -> Router {
    let merged = topic_routes().merge(entity_routes());
    Router::new().nest(VERSION, merged)
}

fn topic_routes() -> Router {
    Router::new()
        .route(concatcp!("/", TOPICS_PATH), get(topics::search))
        .route(concatcp!("/", TOPICS_PATH), post(topics::create))
        .route(formatcp!("/{TOPICS_PATH}/{{topic_id}}"), get(topics::get))
        .route(
            formatcp!("/{TOPICS_PATH}/{{topic_id}}"),
            put(topics::update),
        )
        .route(
            formatcp!("/{TOPICS_PATH}/{{topic_id}}"),
            delete(topics::delete),
        )
}

fn entity_routes() -> Router {
    Router::new()
        .route(
            formatcp!("/{TOPICS_PATH}/{{topic_id}}/{ENTITIES_PATH}"),
            get(entities::search),
        )
        .route(
            formatcp!("/{TOPICS_PATH}/{{topic_id}}/{ENTITIES_PATH}/{{entity_id}}"),
            get(entities::get),
        )
        .route(
            formatcp!("/{TOPICS_PATH}/{{topic_id}}/{ENTITIES_PATH}"),
            post(entities::create),
        )
        .route(
            formatcp!("/{TOPICS_PATH}/{{topic_id}}/{ENTITIES_PATH}/{{entity_id}}"),
            put(entities::update),
        )
        .route(
            formatcp!("/{TOPICS_PATH}/{{topic_id}}/{ENTITIES_PATH}/{{entity_id}}"),
            delete(entities::delete),
        )
}
