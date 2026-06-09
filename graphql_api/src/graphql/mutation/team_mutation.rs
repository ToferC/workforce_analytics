use async_graphql::*;
use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::models::{Team, NewTeam, SkillDomain};
use crate::common_utils::{UserRole, is_operator, RoleGuard};

#[derive(Default)]
pub struct TeamMutation;

#[Object]
impl TeamMutation {

    #[graphql(
        name = "createTeam",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn create_team(
        &self,
        _context: &Context<'_>,
        data: NewTeam,
    ) -> Result<Team> {
        let team = Team::create(&data)?;
        Ok(team)
    }

    #[graphql(
        name = "updateTeam",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn update_team(
        &self,
        _context: &Context<'_>,
        data: TeamData,
    ) -> Result<Team> {
        let mut team = Team::get_by_id(&data.id)?;

        if let Some(s) = data.name_en {
            team.name_en = s;
        };

        if let Some(s) = data.name_fr {
            team.name_fr = s;
        };

        if let Some(s) = data.primary_domain {
            team.primary_domain = s;
        };

        if let Some(s) = data.description_en {
            team.description_en = s;
        };

        if let Some(s) = data.description_fr {
            team.description_fr = s;
        };

        if let Some(s) = data.retired_at {
            team.retired_at = Some(s);
        };

        team.update()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, InputObject)]
#[graphql(complex)]
/// InputObject for Team with Option fields - only include the ones you want to update
pub struct TeamData {
    pub id: Uuid,
    pub name_en: Option<String>,
    pub name_fr: Option<String>,
    pub primary_domain: Option<SkillDomain>,
    pub description_en: Option<String>,
    pub description_fr: Option<String>,
    pub retired_at: Option<NaiveDateTime>,
}
