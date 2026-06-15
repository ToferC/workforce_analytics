use async_graphql::*;
use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::models::{OrgTier, NewOrgTier, SkillDomain};
use crate::common_utils::{UserRole, is_operator, RoleGuard};

#[derive(Default)]
pub struct OrgTierMutation;

#[Object]
impl OrgTierMutation {

    #[graphql(
        name = "createOrgTier",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn create_org_tier(
        &self,
        _context: &Context<'_>,
        data: NewOrgTier,
    ) -> Result<OrgTier> {
        let org_tier = OrgTier::create(&data)?;
        Ok(org_tier)
    }

    #[graphql(
        name = "updateOrgTier",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn update_org_tier(
        &self,
        _context: &Context<'_>,
        data: OrgTierData,
    ) -> Result<OrgTier> {
        let mut org_tier = OrgTier::get_by_id(&data.id)?;

        if let Some(s) = data.name_en {
            org_tier.name_en = s;
        };

        if let Some(s) = data.name_fr {
            org_tier.name_fr = s;
        };

        if let Some(s) = data.tier_level {
            org_tier.tier_level = s;
        };

        if let Some(s) = data.primary_domain {
            org_tier.primary_domain = s;
        };

        if let Some(s) = data.parent_tier {
            org_tier.parent_tier = Some(s);
        };

        if let Some(s) = data.retired_at {
            org_tier.retired_at = Some(s);
        };

        org_tier.update()
    }

    #[graphql(
        name = "restoreOrgTier",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    /// Un-retire an org tier by clearing retired_at.
    pub async fn restore_org_tier(
        &self,
        _context: &Context<'_>,
        id: Uuid,
    ) -> Result<OrgTier> {
        OrgTier::restore(&id)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, InputObject)]
#[graphql(complex)]
/// InputObject for OrgTier with Option fields - only include the ones you want to update
pub struct OrgTierData {
    pub id: Uuid,
    pub name_en: Option<String>,
    pub name_fr: Option<String>,
    pub tier_level: Option<i32>,
    pub primary_domain: Option<SkillDomain>,
    pub parent_tier: Option<Uuid>,
    pub retired_at: Option<NaiveDateTime>,
}
