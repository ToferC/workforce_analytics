use async_graphql::*;
use serde_json::json;
use uuid::Uuid;

use crate::common_utils::{is_operator, RoleGuard, UserRole};
use crate::graphql::authz;
use crate::models::{AuditEvent, NewRoleOffer, Role, RoleOffer, TeamOwnership};

#[derive(Default)]
pub struct RoleOfferMutation;

#[Object]
impl RoleOfferMutation {
    /// Offer a vacant role to a person who currently sits outside the hiring
    /// manager's area. Routed to the candidate's current manager for approval;
    /// the transfer only happens once they accept. For a candidate you already
    /// manage, assign directly instead.
    #[graphql(
        name = "createRoleOffer",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn create_role_offer(
        &self,
        context: &Context<'_>,
        role_id: Uuid,
        person_id: Uuid,
        message: Option<String>,
    ) -> Result<RoleOffer> {
        // The caller must manage the hiring team.
        authz::require_manage_role(context, &role_id)?;

        let target = Role::get_by_id(&role_id)?;
        if !target.active {
            return Err(Error::new("That role is not active"));
        }
        if target.person_id.is_some() {
            return Err(Error::new("That role is already filled; offers target vacant roles"));
        }

        // The candidate must currently hold a role — this is a transfer.
        let current = Role::get_current_for_person(&person_id)?;
        let from_role = current
            .into_iter()
            .next()
            .ok_or_else(|| Error::new("That person holds no current role; assign them directly"))?;

        // If the candidate is already in the caller's managed area, no offer is
        // needed — they can be assigned directly.
        if authz::effective_scope(context)?.manages_team(&from_role.team_id) {
            return Err(Error::new(
                "That person is already in your managed area; assign them directly",
            ));
        }

        // Route approval to the candidate's manager: their explicit reports_to,
        // else their team's owner.
        let approver_role_id = match from_role.reports_to {
            Some(id) => id,
            None => TeamOwnership::get_by_team_id(&from_role.team_id)
                .map(|o| o.owner_role_id)
                .map_err(|_| {
                    Error::new("Could not find a manager to approve this transfer; an admin can assign directly")
                })?,
        };

        // The offer is made on behalf of the hiring team's owner.
        let offered_by_role_id = TeamOwnership::get_by_team_id(&target.team_id)
            .map(|o| o.owner_role_id)
            .map_err(|_| Error::new("The hiring team has no owner to make this offer"))?;

        let offer = RoleOffer::create(&NewRoleOffer {
            role_id,
            person_id,
            offered_by_role_id,
            from_role_id: Some(from_role.id),
            approver_role_id: Some(approver_role_id),
            message,
        })?;

        AuditEvent::log(
            context,
            "offer.create",
            "role_offer",
            Some(offer.id),
            format!("Offered role “{}” to a candidate, pending their manager", target.title_en),
            Some(json!({
                "role_id": role_id,
                "person_id": person_id,
                "from_role_id": from_role.id,
                "approver_role_id": approver_role_id,
            })),
            Some(offer.id),
        );

        Ok(offer)
    }

    /// Accept a pending offer. Executes the transfer atomically and marks the
    /// offer Completed. The caller must manage the candidate's current team.
    #[graphql(
        name = "acceptRoleOffer",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn accept_role_offer(
        &self,
        context: &Context<'_>,
        offer_id: Uuid,
        note: Option<String>,
    ) -> Result<RoleOffer> {
        let offer = RoleOffer::get_by_id(&offer_id)?;
        authorize_decision(context, &offer)?;

        let decided = RoleOffer::accept(&offer_id, offer.approver_role_id, note)?;

        AuditEvent::log(
            context,
            "offer.accept",
            "role_offer",
            Some(decided.id),
            "Accepted transfer offer; candidate moved".to_string(),
            Some(json!({ "role_id": decided.role_id, "person_id": decided.person_id })),
            Some(decided.id),
        );

        Ok(decided)
    }

    /// Decline a pending offer. The caller must manage the candidate's current
    /// team.
    #[graphql(
        name = "declineRoleOffer",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn decline_role_offer(
        &self,
        context: &Context<'_>,
        offer_id: Uuid,
        note: Option<String>,
    ) -> Result<RoleOffer> {
        let offer = RoleOffer::get_by_id(&offer_id)?;
        authorize_decision(context, &offer)?;

        let decided = RoleOffer::decline(&offer_id, offer.approver_role_id, note)?;

        AuditEvent::log(
            context,
            "offer.decline",
            "role_offer",
            Some(decided.id),
            "Declined transfer offer".to_string(),
            None,
            Some(decided.id),
        );

        Ok(decided)
    }

    /// Withdraw a pending offer. The caller must manage the hiring team.
    #[graphql(
        name = "withdrawRoleOffer",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn withdraw_role_offer(
        &self,
        context: &Context<'_>,
        offer_id: Uuid,
        note: Option<String>,
    ) -> Result<RoleOffer> {
        let offer = RoleOffer::get_by_id(&offer_id)?;
        // Only the hiring side can withdraw.
        authz::require_manage_role(context, &offer.role_id)?;

        let decided = RoleOffer::withdraw(&offer_id, note)?;

        AuditEvent::log(
            context,
            "offer.withdraw",
            "role_offer",
            Some(decided.id),
            "Withdrew transfer offer".to_string(),
            None,
            Some(decided.id),
        );

        Ok(decided)
    }
}

/// Accept/decline are the losing manager's call: the caller must manage the
/// candidate's current (from) role's team.
fn authorize_decision(context: &Context<'_>, offer: &RoleOffer) -> Result<()> {
    match offer.from_role_id {
        Some(from_role_id) => authz::require_manage_role(context, &from_role_id),
        // No resolvable from-role (candidate since unassigned): fall back to the
        // approver position's team.
        None => match offer.approver_role_id {
            Some(approver) => authz::require_manage_role(context, &approver),
            None => Err(Error::new("This offer has no manager assigned to decide it")),
        },
    }
}
