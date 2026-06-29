use async_graphql::*;
use uuid::Uuid;

use crate::common_utils::{RoleGuard, UserRole};
use crate::models::AuditEvent;

#[derive(Default)]
pub struct AuditQuery;

#[Object]
impl AuditQuery {
    /// Most recent audit events across the system, newest first. Admin only.
    #[graphql(name = "recentAuditEvents", guard = "RoleGuard::new(UserRole::Admin)")]
    pub async fn recent_audit_events(
        &self,
        _context: &Context<'_>,
        #[graphql(default = 50)] limit: i64,
    ) -> Result<Vec<AuditEvent>> {
        AuditEvent::get_recent(limit)
    }

    /// Audit history for a single entity (e.g. entity_type "role"), newest
    /// first. Admin only.
    #[graphql(name = "auditEventsForEntity", guard = "RoleGuard::new(UserRole::Admin)")]
    pub async fn audit_events_for_entity(
        &self,
        _context: &Context<'_>,
        entity_type: String,
        entity_id: Uuid,
        #[graphql(default = 50)] limit: i64,
    ) -> Result<Vec<AuditEvent>> {
        AuditEvent::get_by_entity(&entity_type, &entity_id, limit)
    }
}
