use anyhow::Result;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<()> {
    prodhugs::init_tracing();

    match dotenvy::dotenv() {
        Ok(path) => tracing::info!(?path, "loaded .env"),
        Err(err) if err.not_found() => {}
        Err(err) => tracing::warn!(%err, "failed to load .env"),
    }

    let cfg = prodhugs::config::Config::from_env()?;
    let app = prodhugs::App::new(cfg).await?;
    app.run(shutdown_signal()).await
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut s) = signal::unix::signal(signal::unix::SignalKind::terminate()) {
            s.recv().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }

    tracing::info!("shutdown signal received");
}
