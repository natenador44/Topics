use axum::{
    Router,
    routing::{delete, get, post, put},
};
use const_format::formatcp;
use utoipa::{OpenApi, openapi};
use utoipa_axum::router::OpenApiRouter;

mod entities;
mod entity_identifiers;
mod topic_sets;
mod topics;

#[derive(OpenApi)]
#[openapi(
    nest((path = formatcp!("{VERSION}{TOPICS_PATH}"), api = topics::ApiDoc))
)]
pub struct ApiDoc;

const VERSION: &str = "/api/v1";
const TOPICS_PATH: &str = "/topics";
const ENTITIES_PATH: &str = formatcp!("{TOPICS_PATH}/{{topic_id}}/entities");
const ENTITY_IDENTIFIERS_PATH: &str = formatcp!("{ENTITIES_PATH}/{{entity_id}}/identifers");
const TOPIC_SETS_PATH: &str = formatcp!("{TOPICS_PATH}/{{topic_id}}/sets");

pub fn routes() -> OpenApiRouter {
    let merged = OpenApiRouter::new()
        .nest(TOPICS_PATH, topics::routes())
        .merge(OpenApiRouter::new().nest(ENTITIES_PATH, entities::routes()))
        .merge(OpenApiRouter::new().nest(ENTITY_IDENTIFIERS_PATH, entity_identifiers::routes()))
        .merge(OpenApiRouter::new().nest(TOPIC_SETS_PATH, topic_sets::routes()));
    OpenApiRouter::new().nest(VERSION, merged)
}
