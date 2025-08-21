use axum::Router;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

mod v1;

pub fn build() -> Router {
    let (router, api) = OpenApiRouter::with_openapi(v1::ApiDoc::openapi())
        .merge(v1::routes())
        .split_for_parts();

    router.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api.clone()))
}
