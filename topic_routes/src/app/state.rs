use crate::app::services::Service;
use axum::extract::FromRef;
use engine::Engine;

#[derive(Clone)]
pub struct AppState<T> {
    pub service: Service<T>,
}

impl<T: Engine> AppState<T> {
    pub fn new(service: Service<T>) -> Self {
        Self { service }
    }
}

impl<T: Engine> FromRef<AppState<T>> for Service<T> {
    fn from_ref(input: &AppState<T>) -> Self {
        input.service.clone()
    }
}
