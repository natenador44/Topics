use error_stack::Result;
use tracing::{error, info};

mod app;
mod error;
mod logging;

use error::InitError;

#[tokio::main]
async fn main() {
    match try_main().await {
        Ok(_) => info!("topic service shutting down"),
        Err(e) => {
            error!("topic service exited with error: {e:?}");
        }
    }
}

async fn try_main() -> Result<(), InitError> {
    logging::init()?;

    app::run().await
}
