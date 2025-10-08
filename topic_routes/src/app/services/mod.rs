mod entities;
mod sets;
mod topics;

pub use entities::EntityService;
pub use sets::SetService;
pub use topics::TopicService;

use crate::error::AppResult;
use engine::Engine;

#[derive(Debug, Clone)]
pub struct Service<T> {
    pub topics: TopicService<T>,
    pub sets: SetService<T>,
    pub entities: EntityService<T>,
}

impl<T> Service<T>
where
    T: Engine + Clone,
{
    pub fn new(engine: T) -> Self {
        Self {
            topics: TopicService::new(engine.clone()),
            sets: SetService::new(engine.clone()),
            entities: EntityService::new(engine),
        }
    }
}

/// Used for service methods that only need to report if
/// a resource exists (or existed), but don't need to get a representation of that data.
/// This can be used by the route layer to return the correct response to the user
#[derive(Debug)]
pub enum ResourceOutcome {
    Found,
    NotFound,
}

#[derive(Debug, thiserror::Error)]
#[error("failed to create service")]
pub struct ServiceBuildError;

pub fn build<T: Engine>(engine: T) -> AppResult<Service<T>, ServiceBuildError> {
    Ok(Service {
        topics: TopicService::new(engine.clone()),
        sets: SetService::new(engine.clone()),
        entities: EntityService::new(engine),
    })
}
