use axum::extract::FromRef;
use crate::service::EntityService;

#[derive(Clone)]
pub struct EntityAppState {
    pub service: EntityService,
}

impl EntityAppState {
    pub fn new(service: EntityService) -> Self {
        Self { service }
    }
}

impl FromRef<EntityAppState> for EntityService {
    fn from_ref(input: &EntityAppState) -> Self {
        input.service.clone()
    }
}
