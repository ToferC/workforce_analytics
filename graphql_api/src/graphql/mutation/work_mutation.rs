use async_graphql::*;
use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::models::{Work, NewWork, Priority, SkillDomain, CapabilityLevel, WorkStatus};
use crate::common_utils::{UserRole, is_operator, RoleGuard};
use crate::graphql::authz;

#[derive(Default)]
pub struct WorkMutation;

#[Object]
impl WorkMutation {

    #[graphql(
        name = "createWork",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn create_work(
        &self,
        context: &Context<'_>,
        data: NewWork,
    ) -> Result<Work> {
        authz::require_manage_task(context, &data.task_id)?;
        let work = Work::create(&data)?;
        Ok(work)
    }

    #[graphql(
        name = "updateWork",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn update_work(
        &self,
        context: &Context<'_>,
        data: WorkData,
    ) -> Result<Work> {
        let mut work = Work::get_by_id(&data.id)?;
        authz::require_manage_task(context, &work.task_id)?;

        // Remember the pre-update status so we can maintain the lifecycle
        // timestamps (started/completed/blocked) after applying the changes.
        let old_status = work.work_status;

        if let Some(s) = data.task_id {
            work.task_id = s;
        };

        // Assign a role (person) identified as having the required
        // capabilities to this work
        if let Some(s) = data.role_id {
            work.role_id = Some(s);
        };

        if let Some(s) = data.work_description {
            work.work_description = s;
        };

        if let Some(s) = data.url {
            work.url = Some(s);
        };

        if let Some(s) = data.domain {
            work.domain = s;
        };

        if let Some(s) = data.skill_id {
            work.skill_id = s;
        };

        if let Some(s) = data.capability_level {
            work.capability_level = s;
        };

        if let Some(s) = data.effort {
            work.effort = s;
        };

        if let Some(s) = data.work_status {
            work.work_status = s;
        };

        if let Some(s) = data.priority {
            work.priority = s;
        };

        // Proposal 1 — user-set due date. Follows the same "None = leave
        // unchanged" convention as every other field on this input, so a
        // partial update (e.g. the assign flow) never wipes it.
        if let Some(d) = data.due_date {
            work.due_date = Some(d);
        };

        // Proposal 2 — blocked context. Set when provided; the block is
        // cleared automatically below when the work leaves BLOCKED.
        if let Some(r) = data.blocked_reason {
            work.blocked_reason = Some(r);
        };
        if let Some(r) = data.blocked_on_role_id {
            work.blocked_on_role_id = Some(r);
        };

        // Maintain server-managed lifecycle timestamps for any status change,
        // and clear blocked context when the work is no longer blocked.
        work.apply_status_transition(old_status);
        if work.work_status != WorkStatus::Blocked {
            work.blocked_reason = None;
            work.blocked_on_role_id = None;
        }

        work.update()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, InputObject)]
#[graphql(complex)]
/// InputObject for Work with Option fields - only include the ones you want to update
pub struct WorkData {
    pub id: Uuid,
    pub task_id: Option<Uuid>,
    pub role_id: Option<Uuid>,
    pub work_description: Option<String>,
    pub url: Option<String>,
    pub domain: Option<SkillDomain>,
    pub skill_id: Option<Uuid>,
    pub capability_level: Option<CapabilityLevel>,
    pub effort: Option<i32>,
    pub work_status: Option<WorkStatus>,
    pub priority: Option<Priority>,
    /// Target completion date (Proposal 1). None leaves it unchanged.
    pub due_date: Option<NaiveDateTime>,
    /// Why the work is blocked (Proposal 2). None leaves it unchanged; it is
    /// cleared automatically when the work leaves BLOCKED.
    pub blocked_reason: Option<String>,
    /// The role this work is waiting on while blocked (Proposal 2). None leaves
    /// it unchanged; cleared automatically when the work leaves BLOCKED.
    pub blocked_on_role_id: Option<Uuid>,
}
