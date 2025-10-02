use crate::error::AppResult;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::error::InitError;

pub fn init() -> AppResult<(), InitError> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_env("TOPICS_LOG"))
        .init();
    // TODO file logging
    Ok(())
}
