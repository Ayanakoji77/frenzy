use crate::{errors::AppError, models::Session, state::AppState};
use axum::{
    Json,
    extract::{Extension, Path, State},
};
use serde_json::{Value, json};

pub async fn list_sessions(
    State(state): State<AppState>,
    Extension(current_user_id): Extension<i64>,
) -> Result<Json<Vec<Session>>, AppError> {
    let sessions = sqlx::query_as!(
        Session,
        "SELECT * FROM sessions WHERE user_id = $1",
        current_user_id
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(sessions))
}

pub async fn revoke_session(
    State(state): State<AppState>,
    Extension(current_user_id): Extension<i64>,
    Path(session_id): Path<i64>,
) -> Result<Json<Value>, AppError> {
    let result = sqlx::query!(
        "DELETE FROM sessions WHERE id = $1 AND user_id = $2",
        session_id,
        current_user_id
    )
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Session not found".into()));
    }

    Ok(Json(json!({"message": "Session revoked"})))
}
