use std::net::SocketAddr;

use anyhow::Context;
use backend::{app, config};
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    init_tracing();

    let app_config = config::AppConfig::from_env().context("failed to load environment config")?;
    let state = app::AppState::bootstrap(app_config)
        .await
        .context("failed to bootstrap app state")?;
    let app = app::build_router(state.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], state.config.app.port));
    let listener = TcpListener::bind(addr).await?;
    info!(address = %addr, "backend listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn init_tracing() {
    let default_level = std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));
    fmt().with_env_filter(filter).json().init();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{SignalKind, signal};

        if let Ok(mut sigterm) = signal(SignalKind::terminate()) {
            let _ = sigterm.recv().await;
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
