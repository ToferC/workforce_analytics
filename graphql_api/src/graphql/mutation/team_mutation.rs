use async_graphql::*;
use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::models::{Team, NewTeam, SkillDomain};
use crate::common_utils::{UserRole, is_operator, RoleGuard};
use crate::graphql::authz;

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
        context: &Context<'_>,
        data: NewTeam,
    ) -> Result<Team> {
        authz::require_manage_tier(context, &data.org_tier_id)?;
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
        context: &Context<'_>,
        data: TeamData,
    ) -> Result<Team> {
        authz::require_manage_team(context, &data.id)?;
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

    #[graphql(
        name = "restoreTeam",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    /// Un-retire a team by clearing retired_at.
    pub async fn restore_team(
        &self,
        context: &Context<'_>,
        id: Uuid,
    ) -> Result<Team> {
        authz::require_manage_team(context, &id)?;
        Team::restore(&id)
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
