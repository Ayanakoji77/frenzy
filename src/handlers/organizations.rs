use crate::handlers::audit::log_action;
use crate::repositories::rbac::has_permission;
use crate::{errors::AppError, models::Organization, state::AppState};
use axum::{
    Json,
    extract::{Extension, Path, Query, State},
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct UpdateOrgPayload {
    pub name: String,
}
#[derive(Deserialize)]
pub struct CreateOrgPayload {
    pub name: String,
}

#[derive(Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub q: Option<String>,
}

#[derive(Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: i64,
}

pub async fn create_organization(
    State(state): State<AppState>,
    Extension(current_user_id): Extension<i64>,
    Json(payload): Json<CreateOrgPayload>,
) -> Result<Json<Organization>, AppError> {
    let mut tx = state.pool.begin().await?;

    let org = sqlx::query_as!(
        Organization,
        "INSERT INTO organizations (name) VALUES ($1) RETURNING id, name, created_at",
        payload.name
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(db_err) = &e {
            if db_err.constraint() == Some("organizations_name_key") {
                return AppError::Conflict("Organization name already exists".into());
            }
        }
        AppError::from(e)
    })?;

    let role_record = sqlx::query!(
        "INSERT INTO roles (organization_id, name) VALUES ($1, $2) RETURNING id",
        org.id,
        "Owner"
    )
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query!(
        "INSERT INTO permissions (role_id, action) VALUES 
        ($1, 'organization:update'), 
        ($1, 'role:create'), 
        ($1, 'permission:assign'),
        ($1, 'organization:update'), 
        ($1, 'role:create'), 
        ($1, 'role:update'), 
        ($1, 'role:delete'), 
        ($1, 'permission:assign'), 
        ($1, 'permission:delete'), 
        ($1, 'member:create'), 
        ($1, 'member:delete')",
        role_record.id
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        "INSERT INTO memberships (user_id, organization_id, role_id) VALUES ($1, $2, $3)",
        current_user_id,
        org.id,
        role_record.id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    log_action(
        &state.pool,
        Some(current_user_id),
        Some(org.id),
        "organization.created",
        &format!("org_name: {}", org.name),
    )
    .await;
    Ok(Json(org))
}

pub async fn list_orgs(
    State(state): State<AppState>,
    Extension(_current_user_id): Extension<i64>,
    Query(params): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<Organization>>, AppError> {
    let limit = params.limit.unwrap_or(10).clamp(1, 100);
    let offset = params.offset.unwrap_or(0);
    let search_term = format!("%{}%", params.q.unwrap_or_default());

    let total_record = sqlx::query!(
        "SELECT count(*) as count FROM organizations WHERE name ILIKE $1",
        search_term
    )
    .fetch_one(&state.pool)
    .await?;

    let total = total_record.count.unwrap_or(0);

    let orgs = sqlx::query_as!(
        Organization,
        r#"
        SELECT id, name, created_at
        FROM organizations
        WHERE name ILIKE $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        search_term,
        limit,
        offset
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(PaginatedResponse { data: orgs, total }))
}

pub async fn get_organization(
    State(state): State<AppState>,
    Extension(_current_user_id): Extension<i64>,
    Path(org_id): Path<i64>,
) -> Result<Json<Organization>, AppError> {
    let org = sqlx::query_as!(
        Organization,
        "SELECT id, name, created_at FROM organizations WHERE id = $1",
        org_id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Organization not found".into()))?;

    Ok(Json(org))
}

pub async fn update_organization(
    State(state): State<AppState>,
    Extension(current_user_id): Extension<i64>,
    Path(org_id): Path<i64>,
    Json(payload): Json<UpdateOrgPayload>,
) -> Result<Json<Organization>, AppError> {
    let is_authorized =
        has_permission(&state.pool, current_user_id, org_id, "organization:update").await?;

    if !is_authorized {
        return Err(AppError::Forbidden(
            "You do not have permission to update this organization".into(),
        ));
    }

    let org = sqlx::query_as!(
        Organization,
        "UPDATE organizations SET name = $1 WHERE id = $2 RETURNING id, name, created_at",
        payload.name,
        org_id
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(org))
}
