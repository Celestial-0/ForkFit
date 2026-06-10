#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use chrono::Utc;
    use uuid::Uuid;
    use crate::common::AppResult;
    use crate::common::id::BackgroundJobId;
    use crate::background::models::{BackgroundJob, NotificationLog};
    use crate::background::repository::BackgroundRepository;

    #[derive(Default)]
    struct MockBackgroundRepository {
        jobs: Arc<Mutex<Vec<BackgroundJob>>>,
        notifications: Arc<Mutex<Vec<NotificationLog>>>,
    }

    impl BackgroundRepository for MockBackgroundRepository {
        async fn enqueue_job(
            &self,
            queue_name: &str,
            job_type: &str,
            payload: serde_json::Value,
        ) -> AppResult<BackgroundJob> {
            let job = BackgroundJob {
                id: BackgroundJobId::new(),
                queue_name: queue_name.to_string(),
                job_type: job_type.to_string(),
                payload,
                status: "queued".to_string(),
                attempts: 0,
                max_attempts: 3,
                run_at: Utc::now(),
                locked_at: None,
                locked_by: None,
                started_at: None,
                completed_at: None,
                error_log: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            self.jobs.lock().unwrap().push(job.clone());
            Ok(job)
        }

        async fn get_next_jobs(
            &self,
            _queue_name: &str,
            limit: i32,
            worker_id: Uuid,
        ) -> AppResult<Vec<BackgroundJob>> {
            let mut jobs = self.jobs.lock().unwrap();
            let mut locked_jobs = Vec::new();
            for job in jobs.iter_mut() {
                if locked_jobs.len() >= limit as usize {
                    break;
                }
                if (job.status == "queued" || job.status == "failed") && job.attempts < job.max_attempts {
                    job.status = "processing".to_string();
                    job.locked_at = Some(Utc::now());
                    job.locked_by = Some(worker_id);
                    job.attempts += 1;
                    locked_jobs.push(job.clone());
                }
            }
            Ok(locked_jobs)
        }

        async fn mark_job_completed(&self, id: BackgroundJobId) -> AppResult<()> {
            let mut jobs = self.jobs.lock().unwrap();
            if let Some(job) = jobs.iter_mut().find(|j| j.id == id) {
                job.status = "completed".to_string();
                job.completed_at = Some(Utc::now());
                job.locked_at = None;
                job.locked_by = None;
            }
            Ok(())
        }

        async fn mark_job_failed(
            &self,
            id: BackgroundJobId,
            error: &str,
            _next_run_in_secs: Option<i64>,
        ) -> AppResult<()> {
            let mut jobs = self.jobs.lock().unwrap();
            if let Some(job) = jobs.iter_mut().find(|j| j.id == id) {
                job.status = "failed".to_string();
                job.locked_at = None;
                job.locked_by = None;
                job.error_log = Some(error.to_string());
            }
            Ok(())
        }

        async fn list_jobs(&self, _page: u64, _per_page: u64) -> AppResult<(Vec<BackgroundJob>, u64)> {
            let jobs = self.jobs.lock().unwrap().clone();
            let total = jobs.len() as u64;
            Ok((jobs, total))
        }

        async fn log_notification(&self, log: NotificationLog) -> AppResult<NotificationLog> {
            self.notifications.lock().unwrap().push(log.clone());
            Ok(log)
        }
    }

    #[tokio::test]
    async fn test_enqueue_and_poll_jobs() {
        let repo = MockBackgroundRepository::default();
        let payload = serde_json::json!({ "recipient": "test@example.com", "subject": "Test", "body": "Hello" });
        
        // 1. Enqueue job
        let job = repo.enqueue_job("default", "send_email", payload).await.unwrap();
        assert_eq!(job.status, "queued");
        assert_eq!(job.job_type, "send_email");

        // 2. Fetch locked jobs
        let worker_id = Uuid::new_v4();
        let fetched = repo.get_next_jobs("default", 5, worker_id).await.unwrap();
        assert_eq!(fetched.len(), 1);
        assert_eq!(fetched[0].id, job.id);
        assert_eq!(fetched[0].status, "processing");
        assert_eq!(fetched[0].locked_by, Some(worker_id));
        assert_eq!(fetched[0].attempts, 1);

        // 3. Mark completed
        repo.mark_job_completed(job.id).await.unwrap();
        let (all_jobs, _) = repo.list_jobs(1, 10).await.unwrap();
        assert_eq!(all_jobs[0].status, "completed");
    }
}
