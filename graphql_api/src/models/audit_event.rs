use async_graphql::*;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde_json::Value;
use uuid::Uuid;

use crate::database::connection;
use crate::models::{Person, Role};
use crate::schema::audit_events;

/// An append-only record of one mutating action taken through the API. The actor
/// is the authenticated principal at the time; `payload` carries a structured
/// before/after (or relevant fields) blob and `correlation_id` ties together the
/// steps of a single workflow (e.g. a transfer offer's transitions).
#[derive(Debug, Clone, Queryable)]
pub struct AuditEvent {
    pub id: Uuid,
    pub occurred_at: NaiveDateTime,
    pub actor_user_id: Option<Uuid>,
    pub actor_role_id: Option<Uuid>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<Uuid>,
    pub summary: Option<String>,
    pub payload: Option<Value>,
    pub correlation_id: Option<Uuid>,
}

#[Object]
impl AuditEvent {
    async fn id(&self) -> Uuid {
        self.id
    }
    async fn occurred_at(&self) -> NaiveDateTime {
        self.occurred_at
    }
    async fn actor_user_id(&self) -> Option<Uuid> {
        self.actor_user_id
    }
    async fn actor_role_id(&self) -> Option<Uuid> {
        self.actor_role_id
    }
    /// Dotted action key, e.g. "role.reports_to.set".
    async fn action(&self) -> &str {
        &self.action
    }
    async fn entity_type(&self) -> &str {
        &self.entity_type
    }
    async fn entity_id(&self) -> Option<Uuid> {
        self.entity_id
    }
    async fn summary(&self) -> Option<String> {
        self.summary.clone()
    }
    /// Structured detail, serialized as a JSON string.
    async fn payload(&self) -> Option<String> {
        self.payload.as_ref().map(|v| v.to_string())
    }
    async fn correlation_id(&self) -> Option<Uuid> {
        self.correlation_id
    }
    /// The actor's position at the time, if it could be resolved.
    async fn actor_role(&self) -> Result<Option<Role>> {
        match self.actor_role_id {
            Some(id) => Ok(Role::get_by_id(&id).ok()),
            None => Ok(None),
        }
    }
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = audit_events)]
pub struct NewAuditEvent {
    pub actor_user_id: Option<Uuid>,
    pub actor_role_id: Option<Uuid>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<Uuid>,
    pub summary: Option<String>,
    pub payload: Option<Value>,
    pub correlation_id: Option<Uuid>,
}

impl AuditEvent {
    fn insert(new: &NewAuditEvent) -> Result<()> {
        let mut conn = connection()?;
        diesel::insert_into(audit_events::table)
            .values(new)
            .execute(&mut conn)?;
        Ok(())
    }

    /// Record an event from a GraphQL context. **Best-effort:** a failure to
    /// write the audit row is logged but never propagated, so auditing can never
    /// break the business operation it describes.
    pub fn log(
        ctx: &Context<'_>,
        action: &str,
        entity_type: &str,
        entity_id: Option<Uuid>,
        summary: impl Into<String>,
        payload: Option<Value>,
        correlation_id: Option<Uuid>,
    ) {
        let (actor_user_id, actor_role_id) = actor_from_ctx(ctx);
        let new = NewAuditEvent {
            actor_user_id,
            actor_role_id,
            action: action.to_string(),
            entity_type: entity_type.to_string(),
            entity_id,
            summary: Some(summary.into()),
            payload,
            correlation_id,
        };
        if let Err(e) = AuditEvent::insert(&new) {
            eprintln!("audit: failed to record {action} on {entity_type}: {e:?}");
        }
    }

    /// Audit history for one entity, most recent first.
    pub fn get_by_entity(entity_type: &str, entity_id: &Uuid, limit: i64) -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let res = audit_events::table
            .filter(audit_events::entity_type.eq(entity_type))
            .filter(audit_events::entity_id.eq(entity_id))
            .order(audit_events::occurred_at.desc())
            .limit(limit)
            .load::<Self>(&mut conn)?;
        Ok(res)
    }

    /// Most recent events across the whole system.
    pub fn get_recent(limit: i64) -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let res = audit_events::table
            .order(audit_events::occurred_at.desc())
            .limit(limit)
            .load::<Self>(&mut conn)?;
        Ok(res)
    }
}

/// Resolve the acting user and their current position from the GraphQL context.
/// The user id is injected into context for authz; the role is best-effort.
fn actor_from_ctx(ctx: &Context<'_>) -> (Option<Uuid>, Option<Uuid>) {
    let user_id = ctx.data_opt::<Uuid>().copied();
    let role_id = user_id.and_then(|uid| {
        Person::get_by_user_id(&uid).ok().and_then(|p| {
            Role::get_current_for_person(&p.id)
                .ok()
                .and_then(|roles| roles.first().map(|r| r.id))
        })
    });
    (user_id, role_id)
}
