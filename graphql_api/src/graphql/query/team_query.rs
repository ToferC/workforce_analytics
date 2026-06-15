use async_graphql::*;

use crate::models::{Team, TeamOwnership};
use uuid::Uuid;

/*
use crate::common_utils::{RoleGuard, is_admin, UserRole};
*/

#[derive(Default)]
pub struct TeamQuery;

#[Object]
impl TeamQuery {

    #[graphql(name = "teamOwnershipByTeamId")]
    /// Returns the ownership record linking a team to its owner. Exposes
    /// the record id so clients can call updateTeamOwnership to reassign
    /// the owner. Errors if the team has no ownership record.
    pub async fn team_ownership_by_team_id(
        &self,
        _context: &Context<'_>,
        team_id: Uuid,
    ) -> Result<TeamOwnership> {
        TeamOwnership::get_by_team_id(&team_id)
    }

    // Teams
    #[graphql(name = "allTeams")]
    /// Returns a vector of all travel groups
    pub async fn all_teams(
        &self, 
        _context: &Context<'_>,
    ) -> Result<Vec<Team>> {

        Team::get_all()
    }

    #[graphql(name = "teamByID")]
    /// Returns a specific travel group by its UUID
    pub async fn team_by_id(
        &self, 
        _context: &Context<'_>,
        id: Uuid
    ) -> Result<Team> {

        Team::get_by_id(&id)
    }

    #[graphql(name = "teamByName")]
    pub async fn team_by_name(
        &self, 
        _context: &Context<'_>,
        name: String,
    ) -> Result<Vec<Team>> {

        Team::get_by_name(name)
    }
}