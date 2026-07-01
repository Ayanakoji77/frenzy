mod config;
mod errors;
mod handlers;
mod middleware;
mod models;
mod repositories;
mod services;
mod state;

use axum::Router;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sqlx=warn,tower_http=debug".into()),
        )
        .init();

    let app_config = config::load_config()?;
    let pool = config::connect_db().await?;

    let state = state::AppState {
        pool,
        config: app_config,
    };

    let app = Router::new()
        .route(
            "/health",
            axum::routing::get(|| async { "Frenzy IAM is Online!" }),
        )
        .with_state(state);

    let server_port = std::env::var("SERVER_PORT").unwrap_or_else(|_| "3000".to_string());
    let server_host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let bind_addr = format!("{}:{}", server_host, server_port);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!("Server listening on http://{}", bind_addr);

    axum::serve(listener, app).await?;

    Ok(())
}
