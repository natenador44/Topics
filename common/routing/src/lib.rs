use std::sync::Arc;

use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use utoipa::{
    PartialSchema,
    openapi::{RefOr, Schema},
};

pub mod error;
pub mod list_criteria;
pub mod pagination;
pub mod stream;

mod auth;
#[cfg(test)]
pub use auth::oauth::{Jwk, Jwks};

mod metrics;
pub mod router;

pub use auth::{
    oauth::OAuthConfig,
    roles::Roles,
    token::{AuthState, validate_token},
};

#[derive(Debug, Clone, Default)]
pub struct ArwLock<T>(Arc<RwLock<T>>);
impl<T> ArwLock<T> {
    pub fn new(data: T) -> Self {
        Self(Arc::new(RwLock::new(data)))
    }
    pub async fn read(&self) -> RwLockReadGuard<'_, T> {
        self.0.read().await
    }

    pub async fn write(&self) -> RwLockWriteGuard<'_, T> {
        self.0.write().await
    }
}

pub fn patch_field_schema() -> impl Into<RefOr<Schema>> {
    <Option<String> as PartialSchema>::schema()
}
