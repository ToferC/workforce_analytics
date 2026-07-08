use async_graphql::*;

use crate::models::{Task, Work, Product, Priority};
use uuid::Uuid;

use crate::common_utils::{RoleGuard, UserRole, is_operator, is_analyst};

/// One task whose priority is inconsistent with the tiers around it
/// (Proposal 7c): either the task itself is ranked below its product, or it
/// has work items ranked below the task. Carries just enough product context
/// to triage the mismatch without loading the full graph per row.
#[derive(SimpleObject)]
pub struct PriorityMismatch {
    pub task_id: Uuid,
    pub task_title: String,
    pub task_priority: Priority,
    pub product_id: Option<Uuid>,
    pub product_name: Option<String>,
    pub product_priority: Option<Priority>,
    /// The task's priority is lower than its parent product's priority.
    pub task_below_product: bool,
    /// Number of work items under this task ranked below the task's priority.
    pub below_work_count: i32,
}

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

    /// Priority-consistency review (Proposal 7c): every task whose priority is
    /// out of step with the tiers around it — either the task is ranked below
    /// its product, or it holds work ranked below the task. Highest task
    /// priority first, so the most urgent inconsistencies (e.g. a CRITICAL
    /// product with LOW work) surface at the top.
    #[graphql(name = "priorityMismatches", guard = "RoleGuard::new(UserRole::Analyst)", visible = "is_analyst")]
    pub async fn priority_mismatches(
        &self,
        _context: &Context<'_>,
    ) -> Result<Vec<PriorityMismatch>> {
        use std::collections::HashMap;

        let tasks = Task::get_all()?;

        // Cache products so we look up each parent at most once.
        let mut product_cache: HashMap<Uuid, Option<Product>> = HashMap::new();

        let mut out = Vec::new();
        for t in tasks {
            let product = match t.product_id {
                Some(pid) => product_cache
                    .entry(pid)
                    .or_insert_with(|| Product::get_by_id(&pid).ok())
                    .clone(),
                None => None,
            };

            let task_below_product = product
                .as_ref()
                .map(|p| t.priority < p.priority)
                .unwrap_or(false);

            let work = Work::get_by_task_id(&t.id)?;
            let below_work_count = work.iter().filter(|w| w.priority < t.priority).count() as i32;

            if task_below_product || below_work_count > 0 {
                out.push(PriorityMismatch {
                    task_id: t.id,
                    task_title: t.title,
                    task_priority: t.priority,
                    product_id: product.as_ref().map(|p| p.id),
                    product_name: product.as_ref().map(|p| p.name_en.clone()),
                    product_priority: product.as_ref().map(|p| p.priority),
                    task_below_product,
                    below_work_count,
                });
            }
        }

        // Highest task priority first (Critical → Low).
        out.sort_by(|a, b| b.task_priority.cmp(&a.task_priority));
        Ok(out)
    }
}