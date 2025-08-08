use axum::Router;
use error_stack::{Result, ResultExt};
use tokio::net::TcpListener;
use tracing::{error, info};

mod error;
mod logging;

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

    let listener = build_listener().await?;

    let routes = build_routes();

    serve_on(listener, routes).await
}

async fn serve_on(listener: TcpListener, routes: Router) -> Result<(), Error> {
    axum::serve(listener, routes)
        .await
        .change_context(Error::InitServe)
}

fn build_routes() -> Router {
    todo!()
}

async fn build_listener() -> Result<TcpListener, Error> {
    TcpListener::bind("0.0.0.0:3000")
        .await
        .change_context(Error::InitPort)
}
