use crate::app::state::AppState;
use const_format::formatcp;
use engine::Engine;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

mod identifiers;
mod sets;
mod topics;

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = formatcp!("{VERSION_PATH}{TOPICS_PATH}"), api = topics::ApiDoc),
        (path = formatcp!("{VERSION_PATH}{IDENTIFIERS_PATH}"), api = identifiers::ApiDoc),
        (path = formatcp!("{VERSION_PATH}{SETS_PATH}"), api = sets::ApiDoc),
    )
)]
pub struct ApiDoc;

const VERSION_PATH: &str = "/api/v1";
const TOPICS_PATH: &str = "/topics";
const IDENTIFIERS_PATH: &str = formatcp!("{TOPICS_PATH}/{{topic_id}}/identifers");
const SETS_PATH: &str = formatcp!("{TOPICS_PATH}/{{topic_id}}/sets");

// see https://docs.rs/axum/latest/axum/struct.Router.html#method.with_state if you're curious why the return type is generic.
// we need to end up with `Router<()>`, and this I guess does that.
pub fn routes<T, S>(app_state: AppState<T>) -> OpenApiRouter<S>
where
    T: Engine + 'static,
{
    let merged = OpenApiRouter::new()
        .nest(TOPICS_PATH, topics::routes())
        .merge(OpenApiRouter::new().nest(IDENTIFIERS_PATH, identifiers::routes()))
        .merge(OpenApiRouter::new().nest(SETS_PATH, sets::routes()));

    OpenApiRouter::new()
        .nest(VERSION_PATH, merged)
        .with_state(app_state)
}
