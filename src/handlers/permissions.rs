use crate::repositories::rbac::has_permission;
use crate::{errors::AppError, models::Permission, state::AppState};
use axum::{
    Json,
    extract::{Extension, Path, State},
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AssignPermissionPayload {
    pub role_id: i64,
    pub action: String,
}

pub async fn list_permissions(
    State(state): State<AppState>,
    Extension(_user_id): Extension<i64>,
) -> Result<Json<Vec<Permission>>, AppError> {
    let perms = sqlx::query_as!(Permission, "SELECT id, role_id, action FROM permissions")
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(perms))
}

pub async fn assign_permission(
    State(state): State<AppState>,
    Extension(current_user_id): Extension<i64>,
    Json(payload): Json<AssignPermissionPayload>,
) -> Result<Json<Permission>, AppError> {
    let role = sqlx::query!(
        "SELECT organization_id FROM roles WHERE id = $1",
        payload.role_id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Role not found".into()))?;

    let is_authorized = has_permission(
        &state.pool,
        current_user_id,
        role.organization_id,
        "permission:assign",
    )
    .await?;

    if !is_authorized {
        return Err(AppError::Forbidden(
            "You lack permission to assign permissions in this organization".into(),
        ));
    }

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
pub async fn remove_permission(
    State(state): State<AppState>,
    Extension(current_user_id): Extension<i64>,
    Path(permission_id): Path<i64>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Complex join to find the org_id related to this specific permission
    let record = sqlx::query!(
        "SELECT r.organization_id FROM permissions p JOIN roles r ON p.role_id = r.id WHERE p.id = $1",
        permission_id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Permission not found".into()))?;

    let is_authorized = has_permission(
        &state.pool,
        current_user_id,
        record.organization_id,
        "permission:delete",
    )
    .await?;
    if !is_authorized {
        return Err(AppError::Forbidden(
            "You lack permission to delete permissions".into(),
        ));
    }

    sqlx::query!("DELETE FROM permissions WHERE id = $1", permission_id)
        .execute(&state.pool)
        .await?;
    Ok(Json(serde_json::json!({"message": "Permission removed"})))
}
