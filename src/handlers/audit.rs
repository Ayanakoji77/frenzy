use sqlx::PgPool;

pub async fn log_action(
    pool: &PgPool,
    actor_id: Option<i64>,
    organization_id: Option<i64>,
    action: &str,
    resource: &str,
) {
    let _ = sqlx::query!(
        "INSERT INTO audit_logs (actor_id, organization_id, action, resource) VALUES ($1, $2, $3, $4)",
        actor_id,
        organization_id,
        action,
        resource
    )
    .execute(pool)
    .await;
}
