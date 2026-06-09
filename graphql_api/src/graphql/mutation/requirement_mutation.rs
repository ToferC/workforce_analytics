use async_graphql::*;
use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::models::{Requirement, NewRequirement, SkillDomain, CapabilityLevel};
use crate::common_utils::{UserRole, is_operator, RoleGuard};

#[derive(Default)]
pub struct RequirementMutation;

#[Object]
impl RequirementMutation {

    #[graphql(
        name = "createRequirement",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn create_requirement(
        &self,
        _context: &Context<'_>,
        data: NewRequirement,
    ) -> Result<Requirement> {
        let requirement = Requirement::create(&data)?;
        Ok(requirement)
    }

    #[graphql(
        name = "updateRequirement",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn update_requirement(
        &self,
        _context: &Context<'_>,
        data: RequirementData,
    ) -> Result<Requirement> {
        let mut requirement = Requirement::get_by_id(&data.id)?;

        if let Some(s) = data.name_en {
            requirement.name_en = s;
        };

        if let Some(s) = data.name_fr {
            requirement.name_fr = s;
        };

        if let Some(s) = data.domain {
            requirement.domain = s;
        };

        if let Some(s) = data.required_level {
            requirement.required_level = s;
        };

        if let Some(s) = data.retired_at {
            requirement.retired_at = Some(s);
        };

        requirement.update()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, InputObject)]
#[graphql(complex)]
/// InputObject for Requirement with Option fields - only include the ones you want to update
pub struct RequirementData {
    pub id: Uuid,
    pub name_en: Option<String>,
    pub name_fr: Option<String>,
    pub domain: Option<SkillDomain>,
    pub required_level: Option<CapabilityLevel>,
    pub retired_at: Option<NaiveDateTime>,
}
