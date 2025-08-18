use axum::{
    Router,
    routing::{delete, get, post, put},
};
use const_format::formatcp;

mod entities;
mod entity_identifiers;
mod topic_sets;
mod topics;

const VERSION: &str = "/v1";
const TOPICS_PATH: &str = "topics";
const ENTITIES_PATH: &str = "entities";
const ENTITY_IDENTIFIERS_PATH: &str = "entityidentifiers";
const TOPIC_SETS_PATH: &str = "sets";

pub fn routes() -> Router {
    let merged = topic_routes()
        .merge(entity_routes())
        .merge(entity_identifier_routes())
        .merge(entity_set_routes());
    Router::new().nest(VERSION, merged)
}

fn topic_routes() -> Router {
    Router::new().nest(
        formatcp!("/{TOPICS_PATH}"),
        Router::new()
            .route("/", get(topics::search))
            .route("/", post(topics::create))
            .route("/{topic_id}", get(topics::get))
            .route("/{topic_id}", put(topics::update))
            .route("/{topic_id}", delete(topics::delete)),
    )
}

fn entity_routes() -> Router {
    Router::new().nest(
        formatcp!("/{TOPICS_PATH}/{{topic_id}}/{ENTITIES_PATH}"),
        Router::new()
            .route("/", get(entities::search))
            .route("/{entity_id}", get(entities::get))
            .route("/", post(entities::create))
            .route("/{entity_id}", put(entities::update))
            .route("/{entity_id}", delete(entities::delete)),
    )
}

fn entity_identifier_routes() -> Router {
    Router::new().nest(
        formatcp!("/{TOPICS_PATH}/{{topic_id}}/{ENTITY_IDENTIFIERS_PATH}"),
        Router::new()
            .route("/", get(entity_identifiers::search))
            .route("/{entity_id}", get(entity_identifiers::get))
            .route("/", post(entity_identifiers::create))
            .route("/{entity_id}", put(entity_identifiers::update))
            .route("/{entity_id}", delete(entity_identifiers::delete)),
    )
}

// subject to change.. not exactly sure of the best way to handle sets yet
fn entity_set_routes() -> Router {
    Router::new().nest(
        formatcp!("/{TOPICS_PATH}/{{topic_id}}/{TOPIC_SETS_PATH}"),
        Router::new()
            .route("/", post(topic_sets::create_empty))
            .route(
                "/{set_id}/entities/{entity_id}",
                put(topic_sets::add_entity_to_set),
            )
            .route(
                "/{set_id}/entityidentifiers/{entity_identifier_id}",
                put(topic_sets::add_entity_identifier_to_set),
            )
            .route("/{set_id}", delete(topic_sets::delete_set))
            .route(
                "/{set_id}/entities/{entity_id}",
                delete(topic_sets::delete_entity_in_set),
            )
            .route(
                "/{set_id}/entityidentifiers/{entity_identifier_id}",
                delete(topic_sets::delete_entity_identifier_in_set),
            ),
    )
}
