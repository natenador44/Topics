use error_stack::Result;
use tracing::{error, info};

mod error;
mod logging;
mod service;

use error::Error;

#[tokio::main]
async fn main() {
    match try_main().await {
        Ok(_) => info!("topic service shutting down"),
        Err(e) => {
            error!("topic service exited with error: {e:?}");
        }
    }
}

async fn try_main() -> Result<(), Error> {
    logging::init()?;

    service::run().await
}
