use crate::services::jwt::generate_access_token;
use crate::{errors::AppError, state::AppState};
use axum::{
    Json,
    extract::{Extension, State},
};
use chrono::{Duration, Utc};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
pub struct RefreshPayload {
    pub refresh_token: String,
}
#[derive(serde::Deserialize)]
pub struct UpdateUserPayload {
    pub username: String,
}
pub async fn get_me(
    State(state): State<AppState>,
    Extension(current_user_id): Extension<i64>,
) -> Result<Json<Value>, AppError> {
    let user = sqlx::query!(
        "SELECT id, username, created_at FROM users WHERE id = $1",
        current_user_id
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(json!({
        "id": user.id,
        "username": user.username,
        "created_at": user.created_at
    })))
}
pub async fn logout(
    State(state): State<AppState>,
    Extension(current_user_id): Extension<i64>,
) -> Result<Json<Value>, AppError> {
    sqlx::query!("DELETE FROM sessions WHERE user_id = $1", current_user_id)
        .execute(&state.pool)
        .await?;

    Ok(Json(json!({"message": "Successfully logged out"})))
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(payload): Json<RefreshPayload>,
) -> Result<Json<Value>, AppError> {
    let session = sqlx::query!(
        "SELECT user_id, expires_at FROM sessions WHERE refresh_token = $1",
        payload.refresh_token
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid refresh token".into()))?;

    if session.expires_at < Utc::now() {
        return Err(AppError::Unauthorized("Refresh token expired".into()));
    }

    let new_access_token = generate_access_token(
        session.user_id,
        &state.config.jwt_secret,
        state.config.jwt_access_expiry_minutes,
    )?;

    Ok(Json(json!({ "access_token": new_access_token })))
}

pub async fn update_profile(
    State(state): State<AppState>,
    Extension(current_user_id): Extension<i64>,
    axum::Json(payload): axum::Json<UpdateUserPayload>,
) -> Result<axum::Json<serde_json::Value>, AppError> {
    let user = sqlx::query!(
        "UPDATE users SET username = $1 WHERE id = $2 RETURNING id, username",
        payload.username,
        current_user_id
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(db_err) = &e {
            if db_err.constraint() == Some("users_username_key") {
                return AppError::Conflict("Username already taken".into());
            }
        }
        AppError::from(e)
    })?;

    Ok(axum::Json(serde_json::json!({
        "message": "Profile updated",
        "user": { "id": user.id, "username": user.username }
    })))
}
