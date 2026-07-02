use crate::errors::AppError;
use sqlx::PgPool;

pub async fn has_permission(
    pool: &PgPool,
    user_id: i64,
    organization_id: i64,
    action: &str,
) -> Result<bool, AppError> {
    let result = sqlx::query!(
        r#"
        SELECT p.id
        FROM permissions p
        JOIN roles r ON p.role_id = r.id
        JOIN memberships m ON m.role_id = r.id
        WHERE m.user_id = $1 
          AND m.organization_id = $2 
          AND p.action = $3
        "#,
        user_id,
        organization_id,
        action
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.is_some())
}
