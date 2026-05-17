//! `prodhugs` backend — Rust port of the original Go service.
//!
//! Layering mirrors the Go version: `http` → `service` → `repo` → `db`.

pub mod cache;
pub mod config;
pub mod crypto;
pub mod db;
pub mod error;
pub mod http;
pub mod jwt;
pub mod metrics;
pub mod models;
pub mod repo;
pub mod service;
pub mod sudoku;
pub mod telegram;
pub mod ws;

use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tracing_subscriber::EnvFilter;

use crate::config::Config;
use crate::http::AppState;
use crate::jwt::JwtManager;
use crate::service::{HugService, NoteService, UserService};
use crate::telegram::{Bot as TgBot, Client as TgClient, LinkStore, LoginStore, Notifier};
use crate::ws::Hub;

pub fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_line_number(true)
        .json()
        .with_current_span(false)
        .with_span_list(false)
        .try_init();
}

pub struct App {
    cfg: Config,
    state: Arc<AppState>,
    background: Vec<JoinHandle<()>>,
}

impl App {
    pub async fn new(cfg: Config) -> Result<Self> {
        cfg.validate_security().context("config validation")?;

        let pool = init_db(&cfg).await?;
        run_migrations(&pool).await?;

        let metrics_handle = crate::metrics::install();

        let jwt = Arc::new(JwtManager::new(
            cfg.jwt.secret.clone(),
            Duration::from_secs(cfg.jwt.access_duration as u64),
            Duration::from_secs(cfg.jwt.refresh_duration as u64),
        ));

        let tg_client = Arc::new(TgClient::new(cfg.telegram.bot_token.clone()));
        let link_store = Arc::new(LinkStore::new());
        let login_store = Arc::new(LoginStore::new());

        let hub = Arc::new(Hub::new(jwt.clone(), metrics_handle.clone()));

        let user_service = Arc::new(UserService::new(
            pool.clone(),
            jwt.clone(),
            link_store.clone(),
            login_store.clone(),
            cfg.telegram.bot_username.clone(),
        ));

        let hug_service = Arc::new(HugService::new(pool.clone(), jwt.clone()));
        let note_service = Arc::new(NoteService::new(pool.clone()));

        let tg_bot = Arc::new(TgBot::new(
            tg_client.clone(),
            link_store.clone(),
            login_store.clone(),
            pool.clone(),
            hug_service.clone(),
            user_service.clone(),
        ));

        let notifier = Arc::new(Notifier::new(
            tg_client.clone(),
            tg_bot.clone(),
            pool.clone(),
        ));

        crate::service::wire_callbacks(
            hug_service.clone(),
            user_service.clone(),
            hub.clone(),
            notifier.clone(),
        );

        let state = Arc::new(AppState {
            cfg: cfg.clone(),
            jwt: jwt.clone(),
            user: user_service.clone(),
            hug: hug_service.clone(),
            note: note_service.clone(),
            hub: hub.clone(),
            login_store: login_store.clone(),
            metrics_handle: metrics_handle.clone(),
        });

        let mut background = Vec::new();
        background.push(spawn_pending_expirer(hug_service.clone()));
        background.push(spawn_intimacy_decay(hug_service.clone()));
        background.push(spawn_vip_tick(user_service.clone()));
        background.push(spawn_telegram_bot(tg_bot.clone()));

        Ok(Self {
            cfg,
            state,
            background,
        })
    }

    pub async fn run<F>(self, shutdown: F) -> Result<()>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let metrics_addr = normalize_addr(&self.cfg.metrics.addr);
        let http_addr = normalize_addr(&self.cfg.http.addr);

        let metrics_state = self.state.clone();
        let metrics_listener = TcpListener::bind(&metrics_addr)
            .await
            .with_context(|| format!("bind metrics listener on {metrics_addr}"))?;
        tracing::info!(addr = %metrics_addr, "metrics server listening");
        let metrics_router = crate::metrics::router(metrics_state);

        let app_router = crate::http::router(self.state.clone());
        let app_listener = TcpListener::bind(&http_addr)
            .await
            .with_context(|| format!("bind app listener on {http_addr}"))?;
        tracing::info!(addr = %http_addr, "http server listening");

        let (tx_shutdown, _rx_shutdown) = tokio::sync::broadcast::channel::<()>(1);
        let tx_a = tx_shutdown.clone();
        let tx_b = tx_shutdown.clone();

        let mut metrics_rx = tx_shutdown.subscribe();
        let metrics_task = tokio::spawn(async move {
            let _ = axum::serve(metrics_listener, metrics_router)
                .with_graceful_shutdown(async move {
                    let _ = metrics_rx.recv().await;
                })
                .await;
        });

        let mut app_rx = tx_shutdown.subscribe();
        let app_task = tokio::spawn(async move {
            let _ = axum::serve(
                app_listener,
                app_router.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .with_graceful_shutdown(async move {
                let _ = app_rx.recv().await;
            })
            .await;
        });

        shutdown.await;
        tracing::info!("shutting down");

        let _ = tx_a.send(());
        let _ = tx_b.send(());

        let _ = tokio::time::timeout(Duration::from_secs(10), async {
            let _ = metrics_task.await;
            let _ = app_task.await;
        })
        .await;

        for handle in self.background {
            handle.abort();
        }

        Ok(())
    }
}

async fn init_db(cfg: &Config) -> Result<sqlx::PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(cfg.postgres.max_conns as u32)
        .min_connections(cfg.postgres.min_conns as u32)
        .max_lifetime(Some(Duration::from_secs(cfg.postgres.max_conn_lifetime as u64)))
        .idle_timeout(Some(Duration::from_secs(5 * 60)))
        .acquire_timeout(Duration::from_secs(10))
        .connect(&cfg.postgres.url)
        .await
        .context("connect to postgres")?;
    Ok(pool)
}

async fn run_migrations(pool: &sqlx::PgPool) -> Result<()> {
    static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
    MIGRATOR.run(pool).await.context("run migrations")
}

fn normalize_addr(addr: &str) -> String {
    if addr.starts_with(':') {
        format!("0.0.0.0{addr}")
    } else {
        addr.to_string()
    }
}

fn spawn_pending_expirer(svc: Arc<HugService>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(5 * 60));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            ticker.tick().await;
            if let Err(err) = svc.expire_pending_hugs().await {
                tracing::error!(%err, "expire_pending_hugs failed");
            }
        }
    })
}

fn spawn_intimacy_decay(svc: Arc<HugService>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(60 * 60));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            ticker.tick().await;
            if let Err(err) = svc.apply_intimacy_decay().await {
                tracing::error!(%err, "intimacy_decay failed");
            }
        }
    })
}

fn spawn_vip_tick(svc: Arc<UserService>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(60));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            ticker.tick().await;
            if let Err(err) = svc.clear_expired_promotions().await {
                tracing::error!(%err, "clear_expired_promotions failed");
            }
        }
    })
}

fn spawn_telegram_bot(bot: Arc<TgBot>) -> JoinHandle<()> {
    tokio::spawn(async move {
        bot.run().await;
    })
}
