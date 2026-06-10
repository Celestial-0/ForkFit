use crate::common::AppResult;
use crate::common::id::BackgroundJobId;
use super::models::{BackgroundJob, NotificationLog};

pub trait BackgroundRepository: Send + Sync {
    async fn enqueue_job(
        &self,
        queue_name: &str,
        job_type: &str,
        payload: serde_json::Value,
    ) -> AppResult<BackgroundJob>;

    async fn get_next_jobs(
        &self,
        queue_name: &str,
        limit: i32,
        worker_id: uuid::Uuid,
    ) -> AppResult<Vec<BackgroundJob>>;

    async fn mark_job_completed(&self, id: BackgroundJobId) -> AppResult<()>;

    async fn mark_job_failed(
        &self,
        id: BackgroundJobId,
        error: &str,
        next_run_in_secs: Option<i64>,
    ) -> AppResult<()>;

    async fn list_jobs(&self, page: u64, per_page: u64) -> AppResult<(Vec<BackgroundJob>, u64)>;

    async fn log_notification(&self, log: NotificationLog) -> AppResult<NotificationLog>;
}
