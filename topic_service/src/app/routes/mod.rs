use axum::Router;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use crate::app::{repository::Repository, state::AppState};

mod response;
mod v1;

pub fn build<T>(app_state: AppState<T>) -> Router
where
    T: Repository + 'static,
{
    let (router, api) = OpenApiRouter::with_openapi(v1::ApiDoc::openapi())
        .merge(v1::routes(app_state))
        .split_for_parts();

    router.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api.clone()))
}
