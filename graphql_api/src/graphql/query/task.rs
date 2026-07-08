use async_graphql::*;

use crate::models::{Task};
use uuid::Uuid;

use crate::common_utils::{RoleGuard, UserRole, is_operator};

#[derive(Default)]
pub struct TaskQuery;

#[Object]
impl TaskQuery {

    // Task
    #[graphql(name = "allTasks", guard = "RoleGuard::new(UserRole::User)")]
    /// Accepts argument of "count" and returns a vector of {count} tasks ordered by
    /// family name.D
    pub async fn all_tasks(
        &self,
        _context: &Context<'_>,
    ) -> Result<Vec<Task>> {

        Task::get_all()
    }

    #[graphql(name = "Tasks", guard = "RoleGuard::new(UserRole::User)")]
    /// Accepts argument of "count" and returns a vector of {count} tasks ordered by
    /// family name
    pub async fn get_tasks(
        &self,
        _context: &Context<'_>,
        count: i64,
    ) -> Result<Vec<Task>> {

        Task::get_count(count)
    }

    #[graphql(name = "taskById", guard = "RoleGuard::new(UserRole::User)")]
    pub async fn task_by_id(
        &self,
        _context: &Context<'_>,
        id: Uuid
    ) -> Result<Task> {

        Task::get_by_id(&id)
    }

    #[graphql(name = "taskByName", guard = "RoleGuard::new(UserRole::User)")]
    pub async fn task_by_name(
        &self,
        _context: &Context<'_>,
        name: String,
    ) -> Result<Vec<Task>> {

        Task::get_by_title(&name)
    }

    /// Approver queue (Proposal 7b): tasks awaiting approval that the caller
    /// manages, newest first. Scoped per row via require_manage_role so an
    /// operator sees only their area and an admin sees all.
    #[graphql(name = "pendingApprovals", guard = "RoleGuard::new(UserRole::Operator)", visible = "is_operator")]
    pub async fn pending_approvals(
        &self,
        context: &Context<'_>,
        limit: Option<i64>,
    ) -> Result<Vec<Task>> {
        let cap = limit.unwrap_or(200).clamp(1, 500);
        let tasks = Task::pending_approvals(cap)?;
        let mut out = Vec::new();
        for t in tasks {
            if crate::graphql::authz::require_manage_role(context, &t.created_by_role_id).is_ok() {
                out.push(t);
            }
        }
        Ok(out)
    }
}