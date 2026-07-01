use async_graphql::*;

use crate::models::{Work, WorkStatus};
use uuid::Uuid;

use crate::common_utils::{RoleGuard, UserRole};

#[derive(Default)]
pub struct WorkQuery;

#[Object]
impl WorkQuery {

    // Works

    #[graphql(name = "work", guard = "RoleGuard::new(UserRole::User)")]
    /// Accepts an argument of "count" and returns a vector of {count} work
    pub async fn get_count_work(&self, _context: &Context<'_>, count: i64) -> Result<Vec<Work>> {

        Work::get_count(count)
    }

    #[graphql(name = "allWork", guard = "RoleGuard::new(UserRole::User)")]
    /// Returns work, filtered and paginated server-side. All arguments are
    /// optional and backward compatible: with none supplied this returns every
    /// work item (the previous behaviour). `status` narrows to one work status;
    /// `unassignedOnly` keeps only work with no assigned role. Pair with
    /// `workCount` for the total under the same filters.
    pub async fn all_works(
        &self,
        _context: &Context<'_>,
        status: Option<WorkStatus>,
        #[graphql(default = false)] unassigned_only: bool,
        limit: Option<i64>,
        #[graphql(default = 0)] offset: i64,
    ) -> Result<Vec<Work>> {
        Work::get_filtered(status, unassigned_only, limit, offset)
    }

    #[graphql(name = "workCount", guard = "RoleGuard::new(UserRole::User)")]
    /// Total number of work items matching the given filters (ignoring
    /// pagination), for driving `allWork` page controls.
    pub async fn work_count(
        &self,
        _context: &Context<'_>,
        status: Option<WorkStatus>,
        #[graphql(default = false)] unassigned_only: bool,
    ) -> Result<i64> {
        Work::count_filtered(status, unassigned_only)
    }

    #[graphql(name = "workById", guard = "RoleGuard::new(UserRole::User)")]
    pub async fn work_by_id(
        &self,
        _context: &Context<'_>,
        id: Uuid,
    ) -> Result<Work> {

        Work::get_by_id(&id)
    }
}