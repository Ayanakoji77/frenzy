use crate::{errors::AppError, state::AppState};
use axum::{
    Json,
    extract::{Extension, State},
};
use serde_json::{Value, json};

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

pub async fn update_profile(
    State(state): State<AppState>,
    Extension(current_user_id): Extension<i64>,
    Json(payload): Json<UpdateUserPayload>,
) -> Result<Json<Value>, AppError> {
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

    Ok(Json(json!({
        "message": "Profile updated",
        "user": { "id": user.id, "username": user.username }
    })))
}
