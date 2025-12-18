use std::time::Instant;

use axum::{
    extract::{MatchedPath, Request},
    middleware::Next,
    response::IntoResponse,
};

const REQUESTS_TOTAL_METRIC_NAME: &str = "http_requests_total";
const REQUEST_DURATION_METRIC_NAME: &str = "http_requests_duration_seconds";

pub async fn track_http(req: Request, next: Next) -> impl IntoResponse {
    // TODO figure out what "matched path" is
    let path = if let Some(matched_path) = req.extensions().get::<MatchedPath>() {
        matched_path.as_str().to_owned()
    } else {
        req.uri().path().to_owned()
    };

    if path.ends_with("metrics") {
        return next.run(req).await;
    }

    let method = req.method().clone();

    let req_size = req
        .headers()
        .get("Content-Length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok());

    if let Some(req_size) = req_size {
        metrics::histogram!("http_request_size").record(req_size as f64);
    }

    let start = Instant::now();
    let response = next.run(req).await;

    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    let labels = [
        ("method", method.to_string()),
        ("path", path),
        ("status", status),
    ];

    metrics::counter!(REQUESTS_TOTAL_METRIC_NAME, &labels).increment(1);

    let histogram = metrics::histogram!(REQUEST_DURATION_METRIC_NAME, &labels);
    histogram.record(latency);
    response
}
