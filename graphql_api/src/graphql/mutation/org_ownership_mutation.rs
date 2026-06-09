use async_graphql::*;
use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::models::{OrgOwnership, NewOrgOwnership};
use crate::common_utils::{UserRole, is_operator, RoleGuard};

#[derive(Default)]
pub struct OrgOwnershipMutation;

#[Object]
impl OrgOwnershipMutation {

    #[graphql(
        name = "createOrgOwnership",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn create_org_ownership(
        &self,
        _context: &Context<'_>,
        data: NewOrgOwnership,
    ) -> Result<OrgOwnership> {
        let org_ownership = OrgOwnership::create(&data)?;
        Ok(org_ownership)
    }

    #[graphql(
        name = "updateOrgOwnership",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn update_org_ownership(
        &self,
        _context: &Context<'_>,
        data: OrgOwnershipData,
    ) -> Result<OrgOwnership> {
        let mut org_ownership = OrgOwnership::get_by_id(data.id)?;

        if let Some(s) = data.owner_id {
            org_ownership.owner_id = s;
        };

        if let Some(s) = data.org_tier_id {
            org_ownership.org_tier_id = s;
        };

        if let Some(s) = data.retired_at {
            org_ownership.retired_at = Some(s);
        };

        org_ownership.update()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, InputObject)]
#[graphql(complex)]
/// InputObject for OrgOwnership with Option fields - only include the ones you want to update
pub struct OrgOwnershipData {
    pub id: Uuid,
    pub owner_id: Option<Uuid>,
    pub org_tier_id: Option<Uuid>,
    pub retired_at: Option<NaiveDateTime>,
}
