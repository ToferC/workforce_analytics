use async_graphql::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::models::{NewSkill, Skill, SkillDomain};
use crate::common_utils::{UserRole,
    is_operator, RoleGuard};
// use rdkafka::producer::FutureProducer;
// use crate::kafka::send_message;

#[derive(Default)]
pub struct SkillMutation;

// Mutation

#[Object]
impl SkillMutation {

    #[graphql(
        name = "createSkill", 
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn create_skill(
        &self,
        _context: &Context<'_>,
        data: NewSkill,
    ) -> Result<Skill> {
        
        let skill = Skill::create(&data)?;

        Ok(skill)
    }

    #[graphql(
        name = "updateSkill", 
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    /// An operator may update a skill.
    /// To update a validated_level, you must include the Uuid of the validator and their 
    /// current validated level. If the validator's level is equal or greater than the 
    /// level they are validating, the system will update validated_level.
    /// Need to update Skill to also track the validator_uuid 
    pub async fn update_skill(
        &self,
        _context: &Context<'_>,
        data: SkillData,
    ) -> Result<Skill> {
        
        let mut skill = Skill::get_by_id(&data.id)?;

        if let Some(s) = data.name_en {
            skill.name_en = s;
        };

        if let Some(s) = data.name_fr {
            skill.name_fr = s;
        };

        if let Some(s) = data.domain {
            skill.domain = s;
        };

        if let Some(s) = data.description_en {
            skill.description_en = s;
        };

        if let Some(s) = data.description_fr {
            skill.description_fr = s;
        };
        
        skill.update()

    }
}

#[derive(Debug, Clone, Deserialize, Serialize, InputObject)]
#[graphql(complex)]
/// InputObject for Skill with Option fields - only include the ones you want to update
/// Skills will be updated rarely once set by the organization
pub struct SkillData {
    pub id: Uuid,
    pub name_en: Option<String>,
    pub name_fr: Option<String>,
    pub domain: Option<SkillDomain>,
    pub description_en: Option<String>,
    pub description_fr: Option<String>,
}