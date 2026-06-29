use async_graphql::*;
use uuid::Uuid;

use crate::common_utils::{RoleGuard, UserRole};
use crate::models::{Person, Role, RoleOffer};

#[derive(Default)]
pub struct RoleOfferQuery;

/// The positions the caller currently occupies, used to scope their offer
/// queues. Empty for principals with no Person/role (admins, agents).
fn caller_role_ids(ctx: &Context<'_>) -> Vec<Uuid> {
    ctx.data_opt::<Uuid>()
        .copied()
        .and_then(|uid| Person::get_by_user_id(&uid).ok())
        .and_then(|p| Role::get_current_for_person(&p.id).ok())
        .map(|roles| roles.into_iter().map(|r| r.id).collect())
        .unwrap_or_default()
}

#[Object]
impl RoleOfferQuery {
    /// Pending transfer offers awaiting my decision (offers for candidates whose
    /// current manager is me).
    #[graphql(name = "incomingRoleOffers", guard = "RoleGuard::new(UserRole::Operator)")]
    pub async fn incoming_role_offers(&self, context: &Context<'_>) -> Result<Vec<RoleOffer>> {
        let ids = caller_role_ids(context);
        if ids.is_empty() {
            return Ok(vec![]);
        }
        RoleOffer::get_incoming_for_approvers(&ids)
    }

    /// Transfer offers I have made (any status).
    #[graphql(name = "outgoingRoleOffers", guard = "RoleGuard::new(UserRole::Operator)")]
    pub async fn outgoing_role_offers(&self, context: &Context<'_>) -> Result<Vec<RoleOffer>> {
        let ids = caller_role_ids(context);
        if ids.is_empty() {
            return Ok(vec![]);
        }
        RoleOffer::get_outgoing_for_offerers(&ids)
    }

    /// A single offer by id.
    #[graphql(name = "roleOfferById", guard = "RoleGuard::new(UserRole::Operator)")]
    pub async fn role_offer_by_id(&self, _context: &Context<'_>, id: Uuid) -> Result<RoleOffer> {
        RoleOffer::get_by_id(&id)
    }
}
