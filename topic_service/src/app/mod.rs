use crate::{
    app::{services::Service, state::AppState},
    error::InitError,
};
use axum::Router;
use error_stack::{Result, ResultExt};
use tokio::net::TcpListener;
use tracing::info;

mod models;
mod pagination;
mod routes;
mod services;
mod state;

pub async fn run() -> Result<(), InitError> {
    let listener = build_listener().await?;

    let services = services::build();
    let app_state = AppState::new(services);

    let routes = routes::build(app_state);

    info!(
        "starting up topic service on port {}",
        listener
            .local_addr()
            .change_context(InitError::Port)?
            .port()
    );

    serve_on(listener, routes).await
}

async fn serve_on(listener: TcpListener, routes: Router) -> Result<(), InitError> {
    axum::serve(listener, routes)
        .await
        .change_context(InitError::Serve)
}

async fn build_listener() -> Result<TcpListener, InitError> {
    TcpListener::bind("0.0.0.0:3000")
        .await
        .change_context(InitError::Port)
}
