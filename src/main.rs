//! miki-payment API entrypoint.
//!
//! Boot order: load `.env` → build config → connect pool → run migrations →
//! serve. The process refuses to start when configuration or the database is
//! unavailable, rather than failing later on the first request.

mod config;
mod db;
mod error;
mod models;
mod routes;
mod state;

use axum::Router;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

use crate::config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config = Config::from_env()?;

    if config.btcpay.set_count() > 0 && !config.btcpay.is_complete() {
        tracing::warn!(
            "BTCPAY_* is only partially set; Bitcoin integration stays disabled"
        );
    }

    let pool = db::connect(&config).await?;
    tracing::info!("database connected");

    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("migrations applied");

    let port = config.server_port;
    let state = state::AppState::new(pool);
    let app: Router = routes::router().with_state(state);

    let listener = TcpListener::bind(("0.0.0.0", port)).await?;
    tracing::info!(port, "listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

/// Resolve on Ctrl-C or SIGTERM so in-flight requests can finish.
async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("shutdown signal received");
}
