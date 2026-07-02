use crate::services::jwt::{generate_access_token, generate_refresh_token};
use crate::{errors::AppError, state::AppState};
use axum::{Json, extract::State};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

#[derive(Deserialize)]
pub struct AuthPayload {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: i64,
    pub username: String,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<AuthPayload>,
) -> Result<Json<UserResponse>, AppError> {
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();
    let hashed_password = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Hashing failed: {}", e)))?
        .to_string();

    let user = sqlx::query!(
        r#"INSERT INTO users (username, password_hash) VALUES ($1, $2) RETURNING id, username"#,
        payload.username,
        hashed_password
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(db_err) = &e {
            if db_err.constraint() == Some("users_username_key") {
                return AppError::Conflict("Username already exists".into());
            }
        }
        AppError::from(e)
    })?;

    Ok(Json(UserResponse {
        id: user.id,
        username: user.username,
    }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<AuthPayload>,
) -> Result<Json<TokenResponse>, AppError> {
    let user = sqlx::query!(
        "SELECT id, password_hash FROM users WHERE username = $1",
        payload.username
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid username or password".into()))?;

    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid hash format: {}", e)))?;

    let is_valid = Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .is_ok();

    if !is_valid {
        return Err(AppError::Unauthorized(
            "Invalid username or password".into(),
        ));
    }

    let access_token = generate_access_token(
        user.id,
        &state.config.jwt_secret,
        state.config.jwt_access_expiry_minutes,
    )?;
    let refresh_token = generate_refresh_token();
    let expires_at = Utc::now() + Duration::days(7);

    sqlx::query!(
        "INSERT INTO sessions (user_id, refresh_token, expires_at) VALUES ($1, $2, $3)",
        user.id,
        refresh_token,
        expires_at
    )
    .execute(&state.pool)
    .await?;

    Ok(Json(TokenResponse {
        access_token,
        refresh_token,
    }))
}
