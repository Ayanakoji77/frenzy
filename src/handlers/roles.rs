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

pub async fn create_role(
    State(state): State<AppState>,
    Extension(_user_id): Extension<i64>, 
    Json(payload): Json<CreateRolePayload>,
) -> Result<Json<Role>, AppError> {
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
