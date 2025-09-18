use crate::error::AppResult;
use crate::{app::state::AppState, error::InitError};
use axum::Router;
use error_stack::ResultExt;
use tokio::net::TcpListener;
use tracing::info;

mod models;
mod pagination;
mod repository;
mod routes;
mod services;
mod state;

#[cfg(test)]
mod tests;

pub async fn run() -> AppResult<(), InitError> {
    let listener = build_listener().await?;

    let services = services::build().change_context(InitError::Service)?; // this error message is going to be redundant, fix later
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

async fn serve_on(listener: TcpListener, routes: Router) -> AppResult<(), InitError> {
    axum::serve(listener, routes)
        .await
        .change_context(InitError::Serve)
}

async fn build_listener() -> AppResult<TcpListener, InitError> {
    TcpListener::bind("0.0.0.0:3000")
        .await
        .change_context(InitError::Port)
}
