#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use chrono::Utc;
    use uuid::Uuid;
    use crate::common::AppResult;
    use crate::common::id::{AuditLogId, UserId, SessionId};
    use crate::audit::models::AuditLog;
    use crate::audit::repository::AuditRepository;

    #[derive(Default)]
    struct MockAuditRepository {
        logs: Arc<Mutex<Vec<AuditLog>>>,
    }

    impl AuditRepository for MockAuditRepository {
        async fn log_audit(&self, log: AuditLog) -> AppResult<AuditLog> {
            self.logs.lock().unwrap().push(log.clone());
            Ok(log)
        }

        async fn list_audit_logs(&self, _page: u64, _per_page: u64) -> AppResult<(Vec<AuditLog>, u64)> {
            let logs = self.logs.lock().unwrap().clone();
            let total = logs.len() as u64;
            Ok((logs, total))
        }
    }

    #[tokio::test]
    async fn test_log_and_list_audit_logs() {
        let repo = MockAuditRepository::default();
        let user_id = UserId::new();
        let session_id = SessionId::new();
        let resource_id = Uuid::new_v4();

        let log = AuditLog {
            id: AuditLogId::new(),
            actor_user_id: Some(user_id),
            session_id: Some(session_id),
            action: "auth.login".to_string(),
            resource_type: "session".to_string(),
            resource_id: Some(resource_id),
            ip_address: Some("127.0.0.1".parse().unwrap()),
            user_agent: Some("Mozilla/5.0".to_string()),
            metadata: serde_json::json!({ "success": true }),
            created_at: Utc::now(),
        };

        // 1. Log audit
        let logged = repo.log_audit(log).await.unwrap();
        assert_eq!(logged.action, "auth.login");
        assert_eq!(logged.actor_user_id, Some(user_id));

        // 2. List audit logs
        let (all_logs, total) = repo.list_audit_logs(1, 10).await.unwrap();
        assert_eq!(total, 1);
        assert_eq!(all_logs[0].id, logged.id);
        assert_eq!(all_logs[0].action, "auth.login");
    }
}
