use async_graphql::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::models::{Work, NewWork, SkillDomain, CapabilityLevel, WorkStatus};
use crate::common_utils::{UserRole, is_operator, RoleGuard};

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
        _context: &Context<'_>,
        data: NewWork,
    ) -> Result<Work> {
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
        _context: &Context<'_>,
        data: WorkData,
    ) -> Result<Work> {
        let mut work = Work::get_by_id(&data.id)?;

        if let Some(s) = data.work_description {
            work.work_description = s;
        };

        if let Some(s) = data.url {
            work.url = Some(s);
        };

        if let Some(s) = data.domain {
            work.domain = s;
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

        work.update()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, InputObject)]
#[graphql(complex)]
/// InputObject for Work with Option fields - only include the ones you want to update
pub struct WorkData {
    pub id: Uuid,
    pub work_description: Option<String>,
    pub url: Option<String>,
    pub domain: Option<SkillDomain>,
    pub capability_level: Option<CapabilityLevel>,
    pub effort: Option<i32>,
    pub work_status: Option<WorkStatus>,
}
