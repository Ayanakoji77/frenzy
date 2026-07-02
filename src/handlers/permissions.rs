use crate::{errors::AppError, models::Permission, state::AppState};
use axum::{
    Json,
    extract::{Extension, State},
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AssignPermissionPayload {
    pub role_id: i64,
    pub action: String, 
}

pub async fn assign_permission(
    State(state): State<AppState>,
    Extension(_user_id): Extension<i64>,
    Json(payload): Json<AssignPermissionPayload>,
) -> Result<Json<Permission>, AppError> {
    let permission = sqlx::query_as!(
        Permission,
        "INSERT INTO permissions (role_id, action) VALUES ($1, $2) RETURNING id, role_id, action",
        payload.role_id,
        payload.action
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(permission))
}
