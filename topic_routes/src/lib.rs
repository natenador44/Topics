use crate::error::AppResult;
use tracing::{error, info};

mod app;
mod error;
// mod logging;

pub use app::*;

// #[tokio::main]
// async fn main() {
//     match try_main().await {
//         Ok(_) => info!("topic service shutting down"),
//         Err(e) => {
//             error!("topic service exited with error: {e:?}");
//         }
//     }
// }
//
// async fn try_main() -> AppResult<(), InitError> {
//     logging::init()?;
//
//     app::run().await
// }
