//! Simple per-IP, per-path token-bucket rate limiter — used for the auth
//! endpoints in the original Go code (register/login/refresh/check-username/etc.)
//!
//! Limits are configured in [`router`](super) when wiring routes.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::{ConnectInfo, MatchedPath, Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use parking_lot::Mutex;

use crate::error::ErrorCode;
use crate::http::error::ApiErrorBody;

#[derive(Clone, Copy, Debug)]
pub struct LimitConfig {
    pub tokens_per_second: f64,
    pub burst: u32,
    pub ttl: Duration,
}

#[derive(Default)]
pub struct LimitTable {
    pub entries: HashMap<String, LimitConfig>,
    pub default: Option<LimitConfig>,
}

#[derive(Default)]
struct Visitor {
    tokens: f64,
    last_seen: Option<Instant>,
}

#[derive(Default)]
pub struct Limiter {
    table: LimitTable,
    visitors: Mutex<HashMap<String, Visitor>>,
}

impl Limiter {
    pub fn new(table: LimitTable) -> Self {
        Self {
            table,
            visitors: Mutex::new(HashMap::new()),
        }
    }

    pub fn check(&self, ip: &str, path: &str) -> bool {
        let Some(cfg) = self
            .table
            .entries
            .get(path)
            .copied()
            .or(self.table.default)
        else {
            return true;
        };
        let key = format!("{ip}:{path}");
        let now = Instant::now();
        let mut guard = self.visitors.lock();
        let entry = guard.entry(key.clone()).or_insert_with(|| Visitor {
            tokens: cfg.burst as f64,
            last_seen: None,
        });
        if let Some(prev) = entry.last_seen {
            let elapsed = now.duration_since(prev).as_secs_f64();
            entry.tokens =
                (entry.tokens + elapsed * cfg.tokens_per_second).min(cfg.burst as f64);
        }
        entry.last_seen = Some(now);
        let allowed = entry.tokens >= 1.0;
        if allowed {
            entry.tokens -= 1.0;
        }
        // Evict stale entries.
        guard.retain(|_, v| {
            v.last_seen
                .map(|t| now.duration_since(t) < cfg.ttl)
                .unwrap_or(false)
        });
        allowed
    }
}

pub async fn middleware(
    State(limiter): State<Arc<Limiter>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    matched: Option<MatchedPath>,
    req: Request,
    next: Next,
) -> Response {
    let path = matched
        .as_ref()
        .map(|p| p.as_str().to_owned())
        .unwrap_or_else(|| req.uri().path().to_string());
    let ip = real_ip(&req, &addr);

    if !limiter.check(&ip, &path) {
        let body = ApiErrorBody {
            code: ErrorCode::RateLimited,
            message: "too many requests, try again later".into(),
        };
        return (StatusCode::TOO_MANY_REQUESTS, axum::Json(body)).into_response();
    }
    next.run(req).await
}

fn real_ip(req: &Request, fallback: &SocketAddr) -> String {
    if let Some(h) = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
    {
        if let Some(first) = h.split(',').next() {
            return first.trim().to_owned();
        }
    }
    if let Some(h) = req
        .headers()
        .get("x-real-ip")
        .and_then(|v| v.to_str().ok())
    {
        return h.to_string();
    }
    fallback.ip().to_string()
}
