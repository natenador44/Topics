use crate::error::Error;
use axum::Router;
use error_stack::{Result, ResultExt};
use tokio::net::TcpListener;
use tracing::info;

mod pagination;
mod routes;

pub async fn run() -> Result<(), Error> {
    let listener = build_listener().await?;

    let routes = routes::build();

    info!(
        "starting up topic service on port {}",
        listener
            .local_addr()
            .change_context(Error::InitPort)?
            .port()
    );

    serve_on(listener, routes).await
}

async fn serve_on(listener: TcpListener, routes: Router) -> Result<(), Error> {
    axum::serve(listener, routes)
        .await
        .change_context(Error::InitServe)
}

async fn build_listener() -> Result<TcpListener, Error> {
    TcpListener::bind("0.0.0.0:3000")
        .await
        .change_context(Error::InitPort)
}
