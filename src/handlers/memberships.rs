use crate::repositories::rbac::has_permission;
use crate::{errors::AppError, state::AppState};
use axum::{
    Json,
    extract::{Extension, Path, State},
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct AddMemberPayload {
    pub user_id: i64,
    pub role_id: i64,
}

#[derive(Serialize)]
pub struct MemberResponse {
    pub user_id: i64,
    pub username: String,
    pub role_id: i64,
    pub role_name: String,
}

pub async fn add_member(
    State(state): State<AppState>,
    Extension(current_user_id): Extension<i64>,
    Path(org_id): Path<i64>,
    Json(payload): Json<AddMemberPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    let is_authorized =
        has_permission(&state.pool, current_user_id, org_id, "member:create").await?;
    if !is_authorized {
        return Err(AppError::Forbidden(
            "You lack permission to add members".into(),
        ));
    }

    sqlx::query!(
        "INSERT INTO memberships (user_id, organization_id, role_id) VALUES ($1, $2, $3)",
        payload.user_id,
        org_id,
        payload.role_id
    )
    .execute(&state.pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(db_err) = &e {
            if db_err.constraint() == Some("memberships_pkey") {
                return AppError::Conflict("User is already a member of this organization".into());
            }
        }
        AppError::from(e)
    })?;

    Ok(Json(
        serde_json::json!({"message": "Member added successfully"}),
    ))
}

pub async fn list_members(
    State(state): State<AppState>,
    Extension(_current_user_id): Extension<i64>,
    Path(org_id): Path<i64>,
) -> Result<Json<Vec<MemberResponse>>, AppError> {
    let members = sqlx::query_as!(
        MemberResponse,
        r#"
        SELECT u.id as user_id, u.username, r.id as role_id, r.name as role_name
        FROM memberships m
        JOIN users u ON m.user_id = u.id
        JOIN roles r ON m.role_id = r.id
        WHERE m.organization_id = $1
        "#,
        org_id
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(members))
}

pub async fn remove_member(
    State(state): State<AppState>,
    Extension(current_user_id): Extension<i64>,
    Path((org_id, target_user_id)): Path<(i64, i64)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let is_authorized =
        has_permission(&state.pool, current_user_id, org_id, "member:delete").await?;
    if !is_authorized {
        return Err(AppError::Forbidden(
            "You lack permission to remove members".into(),
        ));
    }

    sqlx::query!(
        "DELETE FROM memberships WHERE organization_id = $1 AND user_id = $2",
        org_id,
        target_user_id
    )
    .execute(&state.pool)
    .await?;

    Ok(Json(serde_json::json!({"message": "Member removed"})))
}
