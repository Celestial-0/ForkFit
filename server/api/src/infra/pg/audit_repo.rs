use sqlx::PgPool;
use crate::common::AppResult;
use crate::audit::models::AuditLog;
use crate::audit::repository::AuditRepository;

#[derive(Clone, Debug)]
pub struct PgAuditRepository {
    pool: PgPool,
}

impl PgAuditRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl AuditRepository for PgAuditRepository {
    async fn log_audit(&self, log: AuditLog) -> AppResult<AuditLog> {
        let row = sqlx::query_as::<_, AuditLog>(
            r#"
            INSERT INTO audit_logs (actor_user_id, session_id, action, resource_type, resource_id, ip_address, user_agent, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(log.actor_user_id)
        .bind(log.session_id)
        .bind(log.action)
        .bind(log.resource_type)
        .bind(log.resource_id)
        .bind(log.ip_address)
        .bind(log.user_agent)
        .bind(log.metadata)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    async fn list_audit_logs(&self, page: u64, per_page: u64) -> AppResult<(Vec<AuditLog>, u64)> {
        let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM audit_logs")
            .fetch_one(&self.pool)
            .await? as u64;

        let limit = per_page;
        let offset = (page - 1) * per_page;

        let logs = sqlx::query_as::<_, AuditLog>(
            r#"
            SELECT * FROM audit_logs
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok((logs, total))
    }
}
