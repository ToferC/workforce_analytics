use async_graphql::*;

use crate::models::{Work};
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
    /// Returns a vector of all persons ordered by family name
    pub async fn all_works(
        &self,
        _context: &Context<'_>,) -> Result<Vec<Work>> {

        Work::get_all()
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