use crate::app::state::AppState;
use axum::Router;
use engine::Engine;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

mod response;
mod v1;

pub fn build<T>(app_state: AppState<T>) -> Router
where
    T: Engine + 'static,
{
    let (router, api) = OpenApiRouter::with_openapi(v1::ApiDoc::openapi())
        .merge(v1::routes(app_state))
        .split_for_parts();

    router.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api.clone()))
}
