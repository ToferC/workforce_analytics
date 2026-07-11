use async_graphql::*;

use crate::models::{Role};
use uuid::Uuid;

use crate::common_utils::{RoleGuard, UserRole};
use crate::graphql::loaders::off_executor;

#[derive(Default)]
pub struct RoleQuery;

#[Object]
impl RoleQuery {

    // Roles

    #[graphql(name = "activeRoles", guard = "RoleGuard::new(UserRole::User)")]
    /// Accepts an argument of "count" and returns a vector of {count} active role
    pub async fn get_active_role(&self, _context: &Context<'_>, count: i64) -> Result<Vec<Role>> {

        Role::get_active(count)
    }

    #[graphql(name = "vacantRoles", guard = "RoleGuard::new(UserRole::User)")]
    /// Accepts an argument of "count" and returns a vector of {count} active role
    pub async fn get_vacant_role(&self, _context: &Context<'_>, count: i64) -> Result<Vec<Role>> {

        Role::get_vacant(count)
    }

    #[graphql(name = "allRoles", guard = "RoleGuard::new(UserRole::User)")]
    /// Returns a vector of all persons ordered by family name
    /// Active roles, optionally filtered by a search string (titles or
    /// incumbent name), organization, and filled/vacant status, with
    /// limit/offset pagination. No arguments = all active roles, as before.
    pub async fn all_roles(
        &self,
        _context: &Context<'_>,
        search: Option<String>,
        organization_id: Option<Uuid>,
        status: Option<String>,
        limit: Option<i64>,
        #[graphql(default = 0)] offset: i64,
    ) -> Result<Vec<Role>> {
        let search = search.filter(|s| !s.trim().is_empty());
        let status = status.filter(|s| !s.trim().is_empty());
        off_executor(move || {
            Role::get_filtered(search.as_deref(), organization_id, status.as_deref(), limit, offset)
        })
        .await
    }

    /// Total active roles matching the same filters as `allRoles` (ignoring
    /// pagination), for driving page controls.
    #[graphql(name = "rolesCount", guard = "RoleGuard::new(UserRole::User)")]
    pub async fn roles_count(
        &self,
        _context: &Context<'_>,
        search: Option<String>,
        organization_id: Option<Uuid>,
        status: Option<String>,
    ) -> Result<i64> {
        let search = search.filter(|s| !s.trim().is_empty());
        let status = status.filter(|s| !s.trim().is_empty());
        off_executor(move || {
            Role::count_filtered(search.as_deref(), organization_id, status.as_deref())
        })
        .await
    }

    #[graphql(name = "roleById", guard = "RoleGuard::new(UserRole::User)")]
    pub async fn role_by_id(
        &self,
        _context: &Context<'_>,
        id: Uuid,
    ) -> Result<Role> {

        Role::get_by_id(&id)
    }

    #[graphql(name = "roleCount", guard = "RoleGuard::new(UserRole::User)")]
    /// returns a count of the total roles in the system
    pub async fn role_count(
        &self,
        _context: &Context<'_>,
    ) -> Result<i64> {

        Role::count()
    }
}