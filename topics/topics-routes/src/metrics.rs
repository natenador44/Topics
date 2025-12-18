use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};

const TOPICS_RETRIEVED_METRIC_NAME: &str = "topics_retrieved";
const REQUEST_DURATION_METRIC_NAME: &str = "http_requests_duration_seconds";
const REQUEST_SIZE_METRIC_NAME: &str = "http_request_size";

const TOPICS_CREATED_METRIC_NAME: &str = "num_topics_created";

const TOPICS_DELETED_METRIC_NAME: &str = "num_topics_deleted";
const TOPICS_PATCHED_METRIC_NAME: &str = "num_topics_patched";

pub fn setup_recorder() -> PrometheusHandle {
    const EXPONENTIAL_SECONDS: &[f64] = &[0.005, 0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0];

    const REQ_RES_BUCKETS: &[f64] = &[128.0, 256.0, 512.0, 1024.0, 2048.0, 4096.0, 8192.0, 16384.0];

    PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full(REQUEST_DURATION_METRIC_NAME.to_string()),
            EXPONENTIAL_SECONDS,
        )
        .unwrap()
        .set_buckets_for_metric(
            Matcher::Full(REQUEST_SIZE_METRIC_NAME.to_string()),
            REQ_RES_BUCKETS,
        )
        .unwrap()
        .install_recorder()
        .unwrap()
}

#[inline]
pub fn increment_topics_retrieved() {
    increment_topics_retrieved_by(1);
}

#[inline]
#[cfg(not(target_pointer_width = "64"))]
pub fn increment_topics_retrieved_by(amt: usize) {
    match TryInto::<u64>::try_into(amt) {
        Ok(amt) => {
            metrics::counter!(TOPICS_RETRIEVED_METRIC_NAME).increment(amt);
        }
        Err(e) => {
            error!("could not increment topics created metric: {e}");
        }
    }
}

#[inline]
#[cfg(target_pointer_width = "64")]
pub fn increment_topics_retrieved_by(amt: usize) {
    metrics::counter!(TOPICS_RETRIEVED_METRIC_NAME).increment(amt as u64);
}

#[inline]
#[cfg(not(target_pointer_width = "64"))]
pub fn increment_topics_created_by(amt: usize) {
    match TryInto::<u64>::try_into(amt) {
        Ok(amt) => {
            metrics::counter!(TOPICS_CREATED_METRIC_NAME).increment(amt);
        }
        Err(e) => {
            error!("could not increment topics created metric: {e}");
        }
    }
}

#[inline]
#[cfg(target_pointer_width = "64")]
pub fn increment_topics_created_by(amt: usize) {
    metrics::counter!(TOPICS_CREATED_METRIC_NAME).increment(amt as u64);
}

#[inline]
pub fn increment_topics_created() {
    increment_topics_created_by(1);
}

#[inline]
pub fn increment_topics_deleted() {
    metrics::counter!(TOPICS_DELETED_METRIC_NAME).increment(1);
}

#[inline]
pub fn increment_topics_patched() {
    metrics::counter!(TOPICS_PATCHED_METRIC_NAME).increment(1);
}
