use crate::service::TopicService;
use axum::extract::FromRef;
use topics_core::TopicEngine;

#[derive(Clone)]
pub struct TopicAppState<T: TopicEngine> {
    pub service: TopicService<T>,
}

impl<T: TopicEngine> TopicAppState<T> {
    pub fn new(service: TopicService<T>) -> Self {
        Self { service }
    }
}

impl<T: TopicEngine + Clone> FromRef<TopicAppState<T>> for TopicService<T> {
    fn from_ref(input: &TopicAppState<T>) -> Self {
        input.service.clone()
    }
}
