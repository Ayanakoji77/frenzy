use crate::repositories::rbac::has_permission;
use crate::{errors::AppError, models::Role, state::AppState};
use axum::{
    Json,
    extract::{Extension, Path, State},
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateRolePayload {
    pub organization_id: i64,
    pub name: String,
}
#[derive(Deserialize)]
pub struct UpdateRolePayload {
    pub name: String,
}
pub async fn list_roles(
    State(state): State<AppState>,
    Extension(_user_id): Extension<i64>,
) -> Result<Json<Vec<Role>>, AppError> {
    let roles = sqlx::query_as!(Role, "SELECT id, organization_id, name FROM roles")
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(roles))
}

pub async fn create_role(
    State(state): State<AppState>,
    Extension(current_user_id): Extension<i64>,
    Json(payload): Json<CreateRolePayload>,
) -> Result<Json<Role>, AppError> {
    let is_authorized = has_permission(
        &state.pool,
        current_user_id,
        payload.organization_id,
        "role:create",
    )
    .await?;

    if !is_authorized {
        return Err(AppError::Forbidden(
            "You lack permission to create roles in this organization".into(),
        ));
    }

    let role = sqlx::query_as!(
        Role,
        "INSERT INTO roles (organization_id, name) VALUES ($1, $2) RETURNING id, organization_id, name",
                               payload.organization_id,
                               payload.name
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(role))
}

pub async fn update_role(
    State(state): State<AppState>,
    Extension(current_user_id): Extension<i64>,
    Path(role_id): Path<i64>,
    Json(payload): Json<UpdateRolePayload>,
) -> Result<Json<Role>, AppError> {
    let role = sqlx::query!("SELECT organization_id FROM roles WHERE id = $1", role_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Role not found".into()))?;

    let is_authorized = has_permission(
        &state.pool,
        current_user_id,
        role.organization_id,
        "role:update",
    )
    .await?;
    if !is_authorized {
        return Err(AppError::Forbidden(
            "You lack permission to update roles".into(),
        ));
    }

    let updated_role = sqlx::query_as!(
        Role,
        "UPDATE roles SET name = $1 WHERE id = $2 RETURNING id, organization_id, name",
        payload.name,
        role_id
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(updated_role))
}

pub async fn delete_role(
    State(state): State<AppState>,
    Extension(current_user_id): Extension<i64>,
    Path(role_id): Path<i64>,
) -> Result<Json<serde_json::Value>, AppError> {
    let role = sqlx::query!("SELECT organization_id FROM roles WHERE id = $1", role_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Role not found".into()))?;

    let is_authorized = has_permission(
        &state.pool,
        current_user_id,
        role.organization_id,
        "role:delete",
    )
    .await?;
    if !is_authorized {
        return Err(AppError::Forbidden(
            "You lack permission to delete roles".into(),
        ));
    }

    sqlx::query!("DELETE FROM roles WHERE id = $1", role_id)
        .execute(&state.pool)
        .await?;
    Ok(Json(serde_json::json!({"message": "Role deleted"})))
}
