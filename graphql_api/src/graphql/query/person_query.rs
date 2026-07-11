use async_graphql::*;

use crate::models::{Person};
use uuid::Uuid;

use crate::common_utils::{RoleGuard, UserRole};

#[derive(Default)]
pub struct PersonQuery;

#[Object]
impl PersonQuery {

    // People
    #[graphql(name = "allPeople", guard = "RoleGuard::new(UserRole::User)")]
    /// Returns people, filtered and paginated server-side, ordered by family
    /// then given name. All arguments are optional and backward compatible:
    /// with none supplied this returns every non-retired person. `search`
    /// matches given/family name (each term must match one); `organizationId`
    /// and `roleStatus` ("in_role" / "available") narrow further. Pair with
    /// `peopleCount` for the total under the same filters.
    pub async fn all_people(
        &self,
        _context: &Context<'_>,
        search: Option<String>,
        organization_id: Option<Uuid>,
        role_status: Option<String>,
        #[graphql(default = false)] include_retired: bool,
        limit: Option<i64>,
        #[graphql(default = 0)] offset: i64,
    ) -> Result<Vec<Person>> {
        let search = search.filter(|s| !s.trim().is_empty());
        let role_status = role_status.filter(|s| !s.trim().is_empty());
        crate::graphql::loaders::off_executor(move || {
            Person::get_filtered(search.as_deref(), organization_id, role_status.as_deref(), include_retired, limit, offset)
        })
        .await
    }

    #[graphql(name = "People", guard = "RoleGuard::new(UserRole::User)")]
    /// Accepts argument of "count" and returns a vector of {count} persons ordered by
    /// family name
    pub async fn get_people(
        &self,
        _context: &Context<'_>,
        count: i64,
    ) -> Result<Vec<Person>> {

        Person::get_count(count)
    }

    #[graphql(name = "peopleCount", guard = "RoleGuard::new(UserRole::User)")]
    /// Total number of people matching the given filters (ignoring
    /// pagination), for driving `allPeople` page controls. With no arguments
    /// this counts every non-retired person.
    pub async fn people_count(
        &self,
        _context: &Context<'_>,
        search: Option<String>,
        organization_id: Option<Uuid>,
        role_status: Option<String>,
        #[graphql(default = false)] include_retired: bool,
    ) -> Result<i64> {
        let search = search.filter(|s| !s.trim().is_empty());
        let role_status = role_status.filter(|s| !s.trim().is_empty());
        Person::count_filtered(search.as_deref(), organization_id, role_status.as_deref(), include_retired)
    }



    #[graphql(name = "personById", guard = "RoleGuard::new(UserRole::User)")]
    pub async fn person_by_id(
        &self,
        _context: &Context<'_>,
        id: Uuid
    ) -> Result<Person> {

        Person::get_by_id(&id)
    }

    #[graphql(name = "personByName", guard = "RoleGuard::new(UserRole::User)")]
    pub async fn person_by_name(
        &self,
        _context: &Context<'_>,
        name: String,
    ) -> Result<Vec<Person>> {

        Person::get_by_name(&name)
    }
}