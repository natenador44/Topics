use crate::service::TopicService;
use axum::extract::FromRef;
use topics_core::TopicEngine;

#[derive(Clone)]
pub struct TopicAppState<T: TopicEngine> {
    pub service: TopicService<T>,
    pub metrics_enabled: bool,
}

impl<T: TopicEngine> TopicAppState<T> {
    pub fn new_with_metrics(engine: T) -> Self {
        Self { 
            service: TopicService::new(engine),
            metrics_enabled: true,
        }
    }
    
    pub fn new_without_metrics(engine: T) -> Self {
        Self {
            service: TopicService::new(engine),
            metrics_enabled: false,
        }
    }
}

impl<T: TopicEngine + Clone> FromRef<TopicAppState<T>> for TopicService<T> {
    fn from_ref(input: &TopicAppState<T>) -> Self {
        input.service.clone()
    }
}
