use crate::service::TopicService;
use axum::extract::FromRef;

#[derive(Clone)]
pub struct TopicAppState {
    pub service: TopicService,
}

impl TopicAppState {
    pub fn new(service: TopicService) -> Self {
        Self { service }
    }
}

impl FromRef<TopicAppState> for TopicService {
    fn from_ref(input: &TopicAppState) -> Self {
        input.service.clone()
    }
}
