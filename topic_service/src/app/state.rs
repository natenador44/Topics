use axum::extract::FromRef;

use crate::app::{repository::Repository, services::Service};

#[derive(Clone)]
pub struct AppState<T> {
    pub service: Service<T>,
}

impl<T: Repository> AppState<T> {
    pub fn new(service: Service<T>) -> Self {
        Self { service }
    }
}

impl<T: Repository> FromRef<AppState<T>> for Service<T> {
    fn from_ref(input: &AppState<T>) -> Self {
        input.service.clone()
    }
}
