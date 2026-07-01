use async_graphql::*;

use crate::models::{Team, TeamOwnership};
use uuid::Uuid;

use crate::common_utils::{RoleGuard, UserRole};

#[derive(Default)]
pub struct TeamQuery;

#[Object]
impl TeamQuery {

    #[graphql(name = "teamOwnershipByTeamId", guard = "RoleGuard::new(UserRole::User)")]
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
    #[graphql(name = "allTeams", guard = "RoleGuard::new(UserRole::User)")]
    /// Returns teams, filtered and paginated server-side. All arguments are
    /// optional and backward compatible: with none supplied this returns every
    /// non-retired team (the previous behaviour). `search` matches the English
    /// or French name; pass `limit`/`offset` to page. Pair with `teamsCount`
    /// for the total under the same filters.
    pub async fn all_teams(
        &self,
        _context: &Context<'_>,
        search: Option<String>,
        #[graphql(default = false)] include_retired: bool,
        limit: Option<i64>,
        #[graphql(default = 0)] offset: i64,
    ) -> Result<Vec<Team>> {
        let search = search.filter(|s| !s.trim().is_empty());
        Team::get_filtered(search.as_deref(), include_retired, limit, offset)
    }

    #[graphql(name = "teamsCount", guard = "RoleGuard::new(UserRole::User)")]
    /// Total number of teams matching the given filters (ignoring pagination),
    /// for driving `allTeams` page controls.
    pub async fn teams_count(
        &self,
        _context: &Context<'_>,
        search: Option<String>,
        #[graphql(default = false)] include_retired: bool,
    ) -> Result<i64> {
        let search = search.filter(|s| !s.trim().is_empty());
        Team::count_filtered(search.as_deref(), include_retired)
    }

    #[graphql(name = "teamByID", guard = "RoleGuard::new(UserRole::User)")]
    /// Returns a specific travel group by its UUID
    pub async fn team_by_id(
        &self,
        _context: &Context<'_>,
        id: Uuid
    ) -> Result<Team> {

        Team::get_by_id(&id)
    }

    #[graphql(name = "teamByName", guard = "RoleGuard::new(UserRole::User)")]
    pub async fn team_by_name(
        &self,
        _context: &Context<'_>,
        name: String,
    ) -> Result<Vec<Team>> {

        Team::get_by_name(name)
    }
}