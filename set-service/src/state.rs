use axum::extract::FromRef;
use crate::service::SetService;

#[derive(Clone)]
pub struct SetAppState {
    pub service: SetService,
}

impl SetAppState {
    pub fn new(service: SetService) -> Self {
        Self { service }
    }
}

impl FromRef<SetAppState> for SetService {
    fn from_ref(input: &SetAppState) -> Self {
        input.service.clone()
    }
}
