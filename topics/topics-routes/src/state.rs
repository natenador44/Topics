use std::sync::Arc;

use crate::{
    auth::{Jwk, Jwks, OAuthConfig},
    service::TopicService,
};
use axum::extract::FromRef;
use error_stack::{Report, ResultExt};
use tokio::sync::RwLock;
use topics_core::TopicEngine;
use tracing::{debug, error, info, instrument};

#[derive(Clone)]
pub struct TopicAppState<T: TopicEngine> {
    pub service: TopicService<T>,
    pub metrics_enabled: bool,
    pub jwks_keys: Arc<RwLock<Vec<Jwk>>>,
    pub oauth_config: OAuthConfig,
}

pub type StateResult<T> = Result<T, Report<StateErr>>;

#[derive(Debug, thiserror::Error)]
#[error("failed to initialize app state")]
pub struct StateErr;

impl<T: TopicEngine> TopicAppState<T> {
    pub async fn new_with_metrics(engine: T) -> StateResult<Self> {
        Self::new(engine, true).await
    }

    pub async fn new_without_metrics(engine: T) -> StateResult<Self> {
        Self::new(engine, false).await
    }

    #[instrument(skip(engine))]
    async fn new(engine: T, metrics_enabled: bool) -> StateResult<Self> {
        info!("creating new topic state");
        let oauth_config = OAuthConfig::from_env().change_context(StateErr)?;
        debug!("OAuth Config: {oauth_config:?}");
        let jwks = refresh_jwks(&oauth_config.jwks_url)
            .await
            .change_context(StateErr)?;

        Ok(Self {
            service: TopicService::new(engine),
            metrics_enabled,
            jwks_keys: Arc::new(RwLock::new(jwks)),
            oauth_config,
        })
    }

    #[instrument(skip(self))]
    pub async fn find_jwk_key(&self, kid: &str) -> Option<Jwk> {
        let keys = self.jwks_keys.read().await;
        keys.iter().find(|k| k.kid == kid).cloned()
    }
}

#[derive(Debug, thiserror::Error)]
#[error("failed to retrieve jwks data")]
struct JwksErr;

#[instrument]
async fn refresh_jwks(jwks_uri: &str) -> Result<Vec<Jwk>, Report<JwksErr>> {
    info!("fetching JWKS");

    let jwks: Jwks = reqwest::get(jwks_uri)
        .await
        .change_context(JwksErr)?
        .json()
        .await
        .change_context(JwksErr)?;

    if jwks.keys.is_empty() {
        error!("no jwks were found");
    } else {
        info!("found {} jwks", jwks.keys.len());
    }
    Ok(jwks.keys)
}

impl<T: TopicEngine + Clone> FromRef<TopicAppState<T>> for TopicService<T> {
    fn from_ref(input: &TopicAppState<T>) -> Self {
        input.service.clone()
    }
}
