use async_graphql::*;
use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::models::{Organization, NewOrganization};
use crate::common_utils::{UserRole, is_operator, RoleGuard};

#[derive(Default)]
pub struct OrganizationMutation;

#[Object]
impl OrganizationMutation {

    #[graphql(
        name = "createOrganization",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn create_organization(
        &self,
        _context: &Context<'_>,
        data: NewOrganization,
    ) -> Result<Organization> {
        let organization = Organization::create(&data)?;
        Ok(organization)
    }

    #[graphql(
        name = "updateOrganization",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn update_organization(
        &self,
        _context: &Context<'_>,
        data: OrganizationData,
    ) -> Result<Organization> {
        let mut organization = Organization::get_by_id(&data.id)?;

        if let Some(s) = data.name_en {
            organization.name_en = s;
        };

        if let Some(s) = data.name_fr {
            organization.name_fr = s;
        };

        if let Some(s) = data.acronym_en {
            organization.acronym_en = s;
        };

        if let Some(s) = data.acronym_fr {
            organization.acronym_fr = s;
        };

        if let Some(s) = data.org_type {
            organization.org_type = s;
        };

        if let Some(s) = data.url {
            organization.url = s;
        };

        if let Some(s) = data.retired_at {
            organization.retired_at = Some(s);
        };

        organization.update()
    }

    #[graphql(
        name = "restoreOrganization",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    /// Un-retire an organization by clearing retired_at. The update
    /// resolvers treat a null field as "unchanged", so clearing needs a
    /// dedicated mutation.
    pub async fn restore_organization(
        &self,
        _context: &Context<'_>,
        id: Uuid,
    ) -> Result<Organization> {
        Organization::restore(&id)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, InputObject)]
#[graphql(complex)]
/// InputObject for Organization with Option fields - only include the ones you want to update
pub struct OrganizationData {
    pub id: Uuid,
    pub name_en: Option<String>,
    pub name_fr: Option<String>,
    pub acronym_en: Option<String>,
    pub acronym_fr: Option<String>,
    pub org_type: Option<String>,
    pub url: Option<String>,
    pub retired_at: Option<NaiveDateTime>,
}
