use const_format::formatcp;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

mod entities;
mod entity_identifiers;
mod topic_sets;
mod topics;

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = formatcp!("{VERSION_PATH}{TOPICS_PATH}"), api = topics::ApiDoc),
        (path = formatcp!("{VERSION_PATH}{ENTITIES_PATH}"), api = entities::ApiDoc), // TODO I don't think I need these..
        (path = formatcp!("{VERSION_PATH}{ENTITY_IDENTIFIERS_PATH}"), api = entity_identifiers::ApiDoc),
        (path = formatcp!("{VERSION_PATH}{TOPIC_SETS_PATH}"), api = topic_sets::ApiDoc),
    )
)]
pub struct ApiDoc;

const VERSION_PATH: &str = "/api/v1";
const TOPICS_PATH: &str = "/topics";
const ENTITIES_PATH: &str = formatcp!("{TOPICS_PATH}/{{topic_id}}/entities");
const ENTITY_IDENTIFIERS_PATH: &str = formatcp!("{TOPICS_PATH}/{{topic_id}}/identifers");
const TOPIC_SETS_PATH: &str = formatcp!("{TOPICS_PATH}/{{topic_id}}/sets");

pub fn routes() -> OpenApiRouter {
    let merged = OpenApiRouter::new()
        .nest(TOPICS_PATH, topics::routes())
        .merge(OpenApiRouter::new().nest(ENTITIES_PATH, entities::routes()))
        .merge(OpenApiRouter::new().nest(ENTITY_IDENTIFIERS_PATH, entity_identifiers::routes()))
        .merge(OpenApiRouter::new().nest(TOPIC_SETS_PATH, topic_sets::routes()));
    OpenApiRouter::new().nest(VERSION_PATH, merged)
}
