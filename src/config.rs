use anyhow::Context;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub jwt_secret: String,
    pub jwt_access_expiry_minutes: i64,
}

pub fn load_config() -> anyhow::Result<AppConfig> {
    dotenvy::dotenv().ok();

    let jwt_secret = std::env::var("JWT_SECRET").context("jwt secret in env fiole")?;

    let jwt_access_expiry_minutes = std::env::var("JWT_ACCESS_EXPIRY_MINUTES")
        .unwrap_or_else(|_| "15".to_string())
        .parse::<i64>()
        .context("jwt access expiry minutes ")?;

    Ok(AppConfig {
        jwt_secret,
        jwt_access_expiry_minutes,
    })
}

pub async fn connect_db() -> anyhow::Result<PgPool> {
    let url = std::env::var("DATABASE_URL").context("db url present in env")?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&url)
        .await?;

    tracing::info!("database running migrations");
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
