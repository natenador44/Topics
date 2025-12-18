use crate::service::TopicService;
use axum::extract::FromRef;
use error_stack::Report;
use topics_core::TopicEngine;
use tracing::{info, instrument};

#[derive(Clone)]
pub struct TopicAppState<T: TopicEngine> {
    pub service: TopicService<T>,
    pub metrics_enabled: bool,
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
        Ok(Self {
            service: TopicService::new(engine),
            metrics_enabled,
        })
    }
}

impl<T: TopicEngine + Clone> FromRef<TopicAppState<T>> for TopicService<T> {
    fn from_ref(input: &TopicAppState<T>) -> Self {
        input.service.clone()
    }
}

// it would be cool if I could do this for token validation so I don't have to pass this state around
// impl<T: TopicEngine + Clone> FromRef<TopicAppState<T>> for AuthState {
//     fn from_ref(input: &TopicAppState<T>) -> Self {
//         input.validate_token_state.clone()
//     }
// }
