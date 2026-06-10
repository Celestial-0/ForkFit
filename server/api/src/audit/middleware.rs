use std::net::IpAddr;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;
use crate::common::id::{UserId, SessionId, AuditLogId};
use crate::audit::models::AuditLog;
use crate::audit::repository::AuditRepository;
use crate::infra::pg::audit_repo::PgAuditRepository;

pub async fn log_audit(
    db: &PgPool,
    actor_user_id: Option<UserId>,
    session_id: Option<SessionId>,
    action: &str,
    resource_type: &str,
    resource_id: Option<Uuid>,
    ip_address: Option<IpAddr>,
    user_agent: Option<String>,
    metadata: serde_json::Value,
) {
    let repo = PgAuditRepository::new(db.clone());
    
    let ip_network = ip_address.map(|ip| ipnetwork::IpNetwork::from(ip));
    
    let log = AuditLog {
        id: AuditLogId::new(),
        actor_user_id,
        session_id,
        action: action.to_string(),
        resource_type: resource_type.to_string(),
        resource_id,
        ip_address: ip_network,
        user_agent,
        metadata,
        created_at: Utc::now(),
    };

    if let Err(e) = repo.log_audit(log).await {
        tracing::error!(error = %e, "failed_to_persist_audit_log");
    }
}
