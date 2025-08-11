use error_stack::Result;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::error::Error;

pub fn init() -> Result<(), Error> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_env("TOPICS_LOG"))
        .init();
    // TODO file logging
    Ok(())
}
