use crate::error::AppResult;
use crate::{app::state::AppState, error::InitError};
use axum::Router;
use engine::Engine;
use error_stack::ResultExt;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::time::Duration;
use axum::response::Response;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{info, Metadata, Span};

mod routes;
mod services;
mod state;

#[cfg(test)]
mod tests;

pub struct AppProperties {
    pub port: u16,
}

pub async fn run<T: Engine>(engine: T, properties: AppProperties) -> AppResult<(), InitError> {
    let listener = build_listener(properties.port).await?;

    let services = services::build(engine).change_context(InitError::Service)?; // this error message is going to be redundant, fix later
    let app_state = AppState::new(services);

    let routes = routes::build(app_state)
        .layer(ServiceBuilder::new().layer(
            TraceLayer::new_for_http()
                .on_response(|response: &Response, latency: Duration, _span: &Span| {
                    info!("returned {} in {}ms", response.status(), latency.as_millis());
            })
        ));

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

async fn build_listener(port: u16) -> AppResult<TcpListener, InitError> {
    TcpListener::bind(std::net::SocketAddr::V4(SocketAddrV4::new(
        Ipv4Addr::UNSPECIFIED,
        port,
    )))
    .await
    .change_context(InitError::Port)
}
