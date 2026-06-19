use async_graphql::*;
use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::models::{TeamOwnership, NewTeamOwnership};
use crate::common_utils::{UserRole, is_operator, RoleGuard};
use crate::graphql::authz;

#[derive(Default)]
pub struct TeamOwnershipMutation;

#[Object]
impl TeamOwnershipMutation {

    #[graphql(
        name = "createTeamOwnership",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn create_team_ownership(
        &self,
        context: &Context<'_>,
        data: NewTeamOwnership,
    ) -> Result<TeamOwnership> {
        authz::require_manage_team(context, &data.team_id)?;
        let team_ownership = TeamOwnership::create(&data)?;
        Ok(team_ownership)
    }

    #[graphql(
        name = "updateTeamOwnership",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn update_team_ownership(
        &self,
        context: &Context<'_>,
        data: TeamOwnershipData,
    ) -> Result<TeamOwnership> {
        let mut team_ownership = TeamOwnership::get_by_id(&data.id)?;
        authz::require_manage_team(context, &team_ownership.team_id)?;

        if let Some(s) = data.owner_role_id {
            team_ownership.owner_role_id = s;
        };

        if let Some(s) = data.team_id {
            team_ownership.team_id = s;
        };

        if let Some(s) = data.start_datestamp {
            team_ownership.start_datestamp = s;
        };

        if let Some(s) = data.end_date {
            team_ownership.end_date = Some(s);
        };

        team_ownership.update()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, InputObject)]
#[graphql(complex)]
/// InputObject for TeamOwnership with Option fields - only include the ones you want to update
pub struct TeamOwnershipData {
    pub id: Uuid,
    pub owner_role_id: Option<Uuid>,
    pub team_id: Option<Uuid>,
    pub start_datestamp: Option<NaiveDateTime>,
    pub end_date: Option<NaiveDateTime>,
}
