use crate::common::AppResult;
use super::models::AuditLog;

pub trait AuditRepository: Send + Sync {
    async fn log_audit(&self, log: AuditLog) -> AppResult<AuditLog>;
    async fn list_audit_logs(&self, page: u64, per_page: u64) -> AppResult<(Vec<AuditLog>, u64)>;
}
