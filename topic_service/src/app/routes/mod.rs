use axum::Router;

mod v1;

pub fn build() -> Router {
    v1::routes()
}
