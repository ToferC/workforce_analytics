use async_graphql::*;
use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::models::{Task, NewTask, SkillDomain, WorkStatus};
use crate::common_utils::{UserRole, is_operator, RoleGuard};

#[derive(Default)]
pub struct TaskMutation;

#[Object]
impl TaskMutation {

    #[graphql(
        name = "createTask",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn create_task(
        &self,
        _context: &Context<'_>,
        data: NewTask,
    ) -> Result<Task> {
        let task = Task::create(&data)?;
        Ok(task)
    }

    #[graphql(
        name = "updateTask",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn update_task(
        &self,
        _context: &Context<'_>,
        data: TaskData,
    ) -> Result<Task> {
        let mut task = Task::get_by_id(&data.id)?;

        if let Some(s) = data.title {
            task.title = s;
        };

        if let Some(s) = data.domain {
            task.domain = s;
        };

        if let Some(s) = data.intended_outcome {
            task.intended_outcome = s;
        };

        if let Some(s) = data.final_outcome {
            task.final_outcome = Some(s);
        };

        if let Some(s) = data.url {
            task.url = s;
        };

        if let Some(s) = data.approval_tier {
            task.approval_tier = s;
        };

        if let Some(s) = data.start_datestamp {
            task.start_datestamp = s;
        };

        if let Some(s) = data.target_completion_date {
            task.target_completion_date = s;
        };

        if let Some(s) = data.task_status {
            task.task_status = s;
        };

        if let Some(s) = data.completed_date {
            task.completed_date = Some(s);
        };

        task.update()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, InputObject)]
#[graphql(complex)]
/// InputObject for Task with Option fields - only include the ones you want to update
pub struct TaskData {
    pub id: Uuid,
    pub title: Option<String>,
    pub domain: Option<SkillDomain>,
    pub intended_outcome: Option<String>,
    pub final_outcome: Option<String>,
    pub url: Option<String>,
    pub approval_tier: Option<i32>,
    pub start_datestamp: Option<NaiveDateTime>,
    pub target_completion_date: Option<NaiveDateTime>,
    pub task_status: Option<WorkStatus>,
    pub completed_date: Option<NaiveDateTime>,
}
