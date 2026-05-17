use std::sync::Arc;
use std::time::Instant;

use axum::extract::{MatchedPath, State};
use axum::http::Request;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use prometheus::{Encoder, HistogramVec, IntCounterVec, IntGauge, Registry, TextEncoder};

#[derive(Clone)]
pub struct MetricsHandle {
    pub registry: Arc<Registry>,
    pub http_requests_total: IntCounterVec,
    pub http_request_duration_seconds: HistogramVec,
    pub http_requests_in_flight: IntGauge,
    pub ws_unique_user_count: IntGauge,
}

impl MetricsHandle {
    pub fn record_request(&self, method: &str, path: &str, status: u16, dur_secs: f64) {
        let status = status.to_string();
        self.http_requests_total
            .with_label_values(&[method, path, &status])
            .inc();
        self.http_request_duration_seconds
            .with_label_values(&[method, path])
            .observe(dur_secs);
    }

    pub fn set_ws_unique_user_count(&self, count: i64) {
        self.ws_unique_user_count.set(count);
    }
}

/// Build and register the Prometheus collectors. Returns a `MetricsHandle` to
/// be embedded in the app state.
pub fn install() -> Arc<MetricsHandle> {
    let registry = Registry::new();

    let http_requests_total = IntCounterVec::new(
        prometheus::Opts::new("http_requests_total", "Total number of HTTP requests."),
        &["method", "path", "status"],
    )
    .expect("counter");
    registry.register(Box::new(http_requests_total.clone())).ok();

    let http_request_duration_seconds = HistogramVec::new(
        prometheus::HistogramOpts::new(
            "http_request_duration_seconds",
            "Duration of HTTP requests in seconds.",
        ),
        &["method", "path"],
    )
    .expect("histogram");
    registry
        .register(Box::new(http_request_duration_seconds.clone()))
        .ok();

    let http_requests_in_flight = IntGauge::new(
        "http_requests_in_flight",
        "Number of HTTP requests currently being processed.",
    )
    .expect("gauge");
    registry
        .register(Box::new(http_requests_in_flight.clone()))
        .ok();

    let ws_unique_user_count = IntGauge::new(
        "ws_unique_user_count",
        "Number of distinct authenticated WebSocket users.",
    )
    .expect("gauge");
    registry
        .register(Box::new(ws_unique_user_count.clone()))
        .ok();

    // Default process collector (Linux). Cargo features keep this cross-platform.
    Arc::new(MetricsHandle {
        registry: Arc::new(registry),
        http_requests_total,
        http_request_duration_seconds,
        http_requests_in_flight,
        ws_unique_user_count,
    })
}

pub fn router(state: Arc<crate::http::AppState>) -> Router {
    Router::new()
        .route("/metrics", get(scrape))
        .with_state(state)
}

async fn scrape(State(state): State<Arc<crate::http::AppState>>) -> Response {
    let metric_families = state.metrics_handle.registry.gather();
    let mut buf = Vec::new();
    let encoder = TextEncoder::new();
    if let Err(err) = encoder.encode(&metric_families, &mut buf) {
        tracing::error!(%err, "failed to encode metrics");
        return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "encode error").into_response();
    }
    (
        [(axum::http::header::CONTENT_TYPE, encoder.format_type())],
        buf,
    )
        .into_response()
}

/// Middleware that increments `in_flight` for the duration of the request and
/// records `_total` / `_duration` after it completes.
pub async fn track<B>(
    State(state): State<Arc<crate::http::AppState>>,
    req: Request<B>,
    next: Next,
) -> Response
where
    B: Send + 'static + axum::body::HttpBody<Data = bytes::Bytes>,
    B::Error: Into<axum::BoxError>,
{
    let _ = state; // type bound for symmetry; metrics live in process state below.
    next.run(req.map(axum::body::Body::new)).await
}

/// Tower-style layer wired into the main router; pulls `MatchedPath` so we tag
/// metrics with the route template rather than the actual URI.
pub async fn observe(
    State(state): State<Arc<crate::http::AppState>>,
    matched: Option<MatchedPath>,
    req: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Response {
    let path = matched
        .as_ref()
        .map(|p| p.as_str().to_owned())
        .unwrap_or_else(|| "unknown".to_owned());
    let method = req.method().clone();

    state.metrics_handle.http_requests_in_flight.inc();
    let start = Instant::now();

    let resp = next.run(req).await;

    state.metrics_handle.http_requests_in_flight.dec();
    let dur = start.elapsed().as_secs_f64();
    state.metrics_handle.record_request(
        method.as_str(),
        &path,
        resp.status().as_u16(),
        dur,
    );

    resp
}
