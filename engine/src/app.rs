use std::net::{Ipv4Addr, SocketAddrV4};
use std::time::Duration;
use axum::response::Response;
use axum::Router;
use error_stack::{Report, ResultExt};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{info, Span};

pub struct AppProperties {
    pub port: u16,
}

#[derive(Debug, thiserror::Error)]
#[error("the app exited with an error")]
pub struct AppError;

pub type AppResult<T> = Result<T, Report<AppError>>;

// TODO need to split out a lot of this functionality to different crates if I keep
// common stuff like this... feels weird to stuff everything into "engine"

pub async fn run(routes: Router, properties: AppProperties) -> AppResult<()> {
    let listener = build_listener(properties.port).await?;

    let routes = routes.layer(
        ServiceBuilder::new()
            .layer(TraceLayer::new_for_http().on_response(|res: &Response, latency: Duration, _span: &Span| {
                info!(
                    "returned {} in {}ms",
                    res.status(),
                    latency.as_millis()
                );
            })
    ));


    info!(
        "starting up topic service on port {}",
        listener
            .local_addr()
            .change_context(AppError)?
            .port()
    );

    serve_on(listener, routes).await
}

async fn serve_on(listener: TcpListener, routes: Router) -> AppResult<()> {
    axum::serve(listener, routes)
        .await
        .change_context(AppError)
}

async fn build_listener(port: u16) -> AppResult<TcpListener> {
    TcpListener::bind(std::net::SocketAddr::V4(SocketAddrV4::new(
        Ipv4Addr::UNSPECIFIED,
        port,
    )))
        .await
        .change_context(AppError)
}
