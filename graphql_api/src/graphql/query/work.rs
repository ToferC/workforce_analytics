use async_graphql::*;
use chrono::NaiveDateTime;

use crate::models::{Work, WorkStatus, WorkUpdate, Task, User};
use uuid::Uuid;

use crate::common_utils::{RoleGuard, UserRole, is_operator};

/// One open (unresolved) flag with just enough work/task context to triage it
/// from the manager flags queue, without loading the full Work graph per row.
#[derive(SimpleObject)]
pub struct OpenWorkFlag {
    pub update_id: Uuid,
    pub work_id: Uuid,
    pub work_description: String,
    pub task_id: Uuid,
    pub task_title: String,
    pub body: String,
    pub author_name: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Default)]
pub struct WorkQuery;

#[Object]
impl WorkQuery {

    // Works

    #[graphql(name = "work", guard = "RoleGuard::new(UserRole::User)")]
    /// Accepts an argument of "count" and returns a vector of {count} work
    pub async fn get_count_work(&self, _context: &Context<'_>, count: i64) -> Result<Vec<Work>> {

        Work::get_count(count)
    }

    #[graphql(name = "allWork", guard = "RoleGuard::new(UserRole::User)")]
    /// Returns work, filtered and paginated server-side. All arguments are
    /// optional and backward compatible: with none supplied this returns every
    /// work item (the previous behaviour). `status` narrows to one work status;
    /// `unassignedOnly` keeps only work with no assigned role. Pair with
    /// `workCount` for the total under the same filters.
    pub async fn all_works(
        &self,
        _context: &Context<'_>,
        status: Option<WorkStatus>,
        #[graphql(default = false)] unassigned_only: bool,
        limit: Option<i64>,
        #[graphql(default = 0)] offset: i64,
    ) -> Result<Vec<Work>> {
        Work::get_filtered(status, unassigned_only, limit, offset)
    }

    #[graphql(name = "workCount", guard = "RoleGuard::new(UserRole::User)")]
    /// Total number of work items matching the given filters (ignoring
    /// pagination), for driving `allWork` page controls.
    pub async fn work_count(
        &self,
        _context: &Context<'_>,
        status: Option<WorkStatus>,
        #[graphql(default = false)] unassigned_only: bool,
    ) -> Result<i64> {
        Work::count_filtered(status, unassigned_only)
    }

    #[graphql(name = "workById", guard = "RoleGuard::new(UserRole::User)")]
    pub async fn work_by_id(
        &self,
        _context: &Context<'_>,
        id: Uuid,
    ) -> Result<Work> {

        Work::get_by_id(&id)
    }

    /// Manager flags queue (Proposal 3 follow-up): every unresolved FLAG on
    /// work the caller manages, newest first. Scoped per row to the caller's
    /// managed tasks, so an operator sees only their area and an admin sees all.
    #[graphql(name = "openWorkFlags", guard = "RoleGuard::new(UserRole::Operator)", visible = "is_operator")]
    pub async fn open_work_flags(
        &self,
        context: &Context<'_>,
        limit: Option<i64>,
    ) -> Result<Vec<OpenWorkFlag>> {
        let cap = limit.unwrap_or(200).clamp(1, 500);
        let flags = WorkUpdate::open_flags(cap)?;

        let mut out = Vec::new();
        for f in flags {
            let work = match Work::get_by_id(&f.work_id) {
                Ok(w) => w,
                Err(_) => continue,
            };
            // Only surface flags on work under a task the caller manages.
            if crate::graphql::authz::require_manage_task(context, &work.task_id).is_err() {
                continue;
            }
            let task_title = Task::get_by_id(&work.task_id).map(|t| t.title).unwrap_or_default();
            let author_name = f.author_user_id
                .and_then(|id| User::get_by_id(&id).ok().map(|u| u.name));
            out.push(OpenWorkFlag {
                update_id: f.id,
                work_id: work.id,
                work_description: work.work_description,
                task_id: work.task_id,
                task_title,
                body: f.body,
                author_name,
                created_at: f.created_at,
            });
        }
        Ok(out)
    }
}