use sqlx::PgPool;
use crate::common::AppResult;
use crate::common::id::BackgroundJobId;
use crate::background::models::{BackgroundJob, NotificationLog};
use crate::background::repository::BackgroundRepository;

#[derive(Clone, Debug)]
pub struct PgBackgroundRepository {
    pool: PgPool,
}

impl PgBackgroundRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl BackgroundRepository for PgBackgroundRepository {
    async fn enqueue_job(
        &self,
        queue_name: &str,
        job_type: &str,
        payload: serde_json::Value,
    ) -> AppResult<BackgroundJob> {
        let row = sqlx::query_as::<_, BackgroundJob>(
            r#"
            INSERT INTO background_jobs (queue_name, job_type, payload, status)
            VALUES ($1, $2, $3, 'queued')
            RETURNING *
            "#,
        )
        .bind(queue_name)
        .bind(job_type)
        .bind(payload)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    async fn get_next_jobs(
        &self,
        queue_name: &str,
        limit: i32,
        worker_id: uuid::Uuid,
    ) -> AppResult<Vec<BackgroundJob>> {
        let rows = sqlx::query_as::<_, BackgroundJob>(
            r#"
            UPDATE background_jobs
            SET status = 'processing',
                locked_at = NOW(),
                locked_by = $1,
                started_at = NOW(),
                attempts = attempts + 1,
                updated_at = NOW()
            WHERE id IN (
                SELECT id
                FROM background_jobs
                WHERE queue_name = $2
                  AND status IN ('queued', 'failed')
                  AND attempts < max_attempts
                  AND run_at <= NOW()
                  AND (locked_at IS NULL OR locked_at < NOW() - INTERVAL '5 minutes')
                ORDER BY run_at ASC, created_at ASC
                LIMIT $3
                FOR UPDATE SKIP LOCKED
            )
            RETURNING *
            "#,
        )
        .bind(worker_id)
        .bind(queue_name)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    async fn mark_job_completed(&self, id: BackgroundJobId) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE background_jobs
            SET status = 'completed',
                completed_at = NOW(),
                locked_at = NULL,
                locked_by = NULL,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn mark_job_failed(
        &self,
        id: BackgroundJobId,
        error: &str,
        next_run_in_secs: Option<i64>,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE background_jobs
            SET status = 'failed',
                locked_at = NULL,
                locked_by = NULL,
                error_log = $2,
                run_at = CASE 
                    WHEN $3::bigint IS NOT NULL THEN NOW() + ($3 * INTERVAL '1 second')
                    ELSE NOW() + (attempts * INTERVAL '1 minute')
                END,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(error)
        .bind(next_run_in_secs)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn list_jobs(&self, page: u64, per_page: u64) -> AppResult<(Vec<BackgroundJob>, u64)> {
        let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM background_jobs")
            .fetch_one(&self.pool)
            .await? as u64;

        let limit = per_page;
        let offset = (page - 1) * per_page;

        let jobs = sqlx::query_as::<_, BackgroundJob>(
            r#"
            SELECT * FROM background_jobs
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok((jobs, total))
    }

    async fn log_notification(&self, log: NotificationLog) -> AppResult<NotificationLog> {
        let row = sqlx::query_as::<_, NotificationLog>(
            r#"
            INSERT INTO notification_logs (user_id, channel, recipient, status, provider, provider_message_id, error_message, sent_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(log.user_id)
        .bind(log.channel)
        .bind(log.recipient)
        .bind(log.status)
        .bind(log.provider)
        .bind(log.provider_message_id)
        .bind(log.error_message)
        .bind(log.sent_at)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }
}
