//! HTTP transport — axum routers, shared state, middlewares.

pub mod auth;
pub mod dto;
pub mod error;
pub mod ratelimit;
pub mod swagger;
pub mod v1;
pub mod v2;

use std::sync::Arc;

use axum::middleware::from_fn_with_state;
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::config::Config;
use crate::jwt::JwtManager;
use crate::metrics::MetricsHandle;
use crate::service::{HugService, NoteService, UserService};
use crate::telegram::LoginStore;
use crate::ws::Hub;

/// Shared state exposed to all handlers.
pub struct AppState {
    pub cfg: Config,
    pub jwt: Arc<JwtManager>,
    pub user: Arc<UserService>,
    pub hug: Arc<HugService>,
    pub note: Arc<NoteService>,
    pub hub: Arc<Hub>,
    pub login_store: Arc<LoginStore>,
    pub metrics_handle: Arc<MetricsHandle>,
}

/// Build the main `axum::Router` (the API + WebSocket endpoint).
pub fn router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let v1_routes = v1::router(state.clone());
    let v2_routes = v2::router(state.clone());

    Router::new()
        .nest("/api/v1", v1_routes)
        .nest("/api/v2", v2_routes)
        .route("/api/v1/ws", get(ws_endpoint))
        .merge(swagger::router(state.clone()))
        .with_state(state.clone())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(from_fn_with_state(state.clone(), crate::metrics::observe))
}

async fn ws_endpoint(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    ws: axum::extract::ws::WebSocketUpgrade,
) -> Response {
    let hub = state.hub.clone();
    ws.on_upgrade(move |socket| async move { hub.handle_socket(socket).await })
}
