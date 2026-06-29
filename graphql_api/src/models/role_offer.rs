use async_graphql::*;
use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use diesel::Connection;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::connection;
use crate::models::{Person, Role};
use crate::schema::{role_assignments, role_offers, roles};

/// Lifecycle of a transfer offer.
///
/// ```text
/// Pending --accept--> (transfer runs) --> Completed
/// Pending --decline--> Declined
/// Pending --withdraw--> Withdrawn
/// Pending --expire--> Expired
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, DbEnum, Serialize, Deserialize, Enum)]
#[ExistingTypePath = "crate::schema::sql_types::RoleOfferStatus"]
pub enum RoleOfferStatus {
    Pending,
    Accepted,
    Declined,
    Withdrawn,
    Expired,
    Completed,
}

/// An offer to move a person into a (vacant) role, pending the approval of the
/// person's current manager. See `docs/ORG_WORKFLOWS_DESIGN.md`.
#[derive(Debug, Clone, Queryable, Identifiable)]
#[diesel(table_name = role_offers)]
pub struct RoleOffer {
    pub id: Uuid,
    pub role_id: Uuid,
    pub person_id: Uuid,
    pub offered_by_role_id: Uuid,
    pub from_role_id: Option<Uuid>,
    pub approver_role_id: Option<Uuid>,
    pub status: RoleOfferStatus,
    pub message: Option<String>,
    pub decision_note: Option<String>,
    pub decided_by_role_id: Option<Uuid>,
    pub decided_at: Option<NaiveDateTime>,
    pub expires_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[Object]
impl RoleOffer {
    async fn id(&self) -> Uuid {
        self.id
    }
    async fn status(&self) -> RoleOfferStatus {
        self.status
    }
    async fn message(&self) -> Option<String> {
        self.message.clone()
    }
    async fn decision_note(&self) -> Option<String> {
        self.decision_note.clone()
    }
    async fn created_at(&self) -> NaiveDateTime {
        self.created_at
    }
    async fn updated_at(&self) -> NaiveDateTime {
        self.updated_at
    }
    async fn decided_at(&self) -> Option<NaiveDateTime> {
        self.decided_at
    }
    async fn expires_at(&self) -> Option<NaiveDateTime> {
        self.expires_at
    }
    /// The offered (target) position.
    async fn role(&self) -> Result<Role> {
        Role::get_by_id(&self.role_id)
    }
    /// The candidate.
    async fn person(&self) -> Result<Person> {
        Person::get_by_id(&self.person_id)
    }
    /// The hiring manager's position that made the offer.
    async fn offered_by_role(&self) -> Result<Role> {
        Role::get_by_id(&self.offered_by_role_id)
    }
    /// The candidate's role at the time of the offer.
    async fn from_role(&self) -> Result<Option<Role>> {
        Ok(self.from_role_id.and_then(|id| Role::get_by_id(&id).ok()))
    }
    /// The position responsible for approving (the losing manager).
    async fn approver_role(&self) -> Result<Option<Role>> {
        Ok(self.approver_role_id.and_then(|id| Role::get_by_id(&id).ok()))
    }
    /// The position that decided the offer, once decided.
    async fn decided_by_role(&self) -> Result<Option<Role>> {
        Ok(self.decided_by_role_id.and_then(|id| Role::get_by_id(&id).ok()))
    }
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = role_offers)]
pub struct NewRoleOffer {
    pub role_id: Uuid,
    pub person_id: Uuid,
    pub offered_by_role_id: Uuid,
    pub from_role_id: Option<Uuid>,
    pub approver_role_id: Option<Uuid>,
    pub message: Option<String>,
}

impl RoleOffer {
    pub fn create(new: &NewRoleOffer) -> Result<Self> {
        let mut conn = connection()?;
        let res = diesel::insert_into(role_offers::table)
            .values(new)
            .get_result(&mut conn)?;
        Ok(res)
    }

    pub fn get_by_id(id: &Uuid) -> Result<Self> {
        let mut conn = connection()?;
        let res = role_offers::table.filter(role_offers::id.eq(id)).first(&mut conn)?;
        Ok(res)
    }

    /// Pending offers awaiting one of these approver positions — a manager's
    /// "incoming" queue.
    pub fn get_incoming_for_approvers(approver_role_ids: &[Uuid]) -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let res = role_offers::table
            .filter(role_offers::status.eq(RoleOfferStatus::Pending))
            .filter(role_offers::approver_role_id.eq_any(approver_role_ids))
            .order(role_offers::created_at.desc())
            .load::<Self>(&mut conn)?;
        Ok(res)
    }

    /// Offers made by one of these positions (any status) — a manager's
    /// "outgoing" queue.
    pub fn get_outgoing_for_offerers(offerer_role_ids: &[Uuid]) -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let res = role_offers::table
            .filter(role_offers::offered_by_role_id.eq_any(offerer_role_ids))
            .order(role_offers::created_at.desc())
            .load::<Self>(&mut conn)?;
        Ok(res)
    }

    /// Accept the offer and execute the transfer **atomically**: re-checks that
    /// the candidate hasn't already moved and the target is still vacant, runs
    /// the assignment, and marks the offer Completed — all in one transaction,
    /// so the offer state and the assignment can never diverge.
    pub fn accept(offer_id: &Uuid, decider_role_id: Option<Uuid>, note: Option<String>) -> Result<Self> {
        let mut conn = connection()?;
        let now = Utc::now().naive_utc();

        // The transaction returns Ok(Err(msg)) for a business rejection (no
        // writes have happened yet, so committing is a no-op) and propagates
        // real DB errors via `?` to roll back.
        let outcome = conn.transaction::<std::result::Result<RoleOffer, String>, diesel::result::Error, _>(|conn| {
            let offer: RoleOffer = match role_offers::table.filter(role_offers::id.eq(offer_id)).first(conn) {
                Ok(o) => o,
                Err(_) => return Ok(Err("Offer not found".to_string())),
            };

            if offer.status != RoleOfferStatus::Pending {
                return Ok(Err("This offer is no longer pending".to_string()));
            }

            // Staleness: the candidate must still hold the role the offer was
            // made from.
            let current_role: Option<Uuid> = role_assignments::table
                .filter(role_assignments::person_id.eq(offer.person_id))
                .filter(role_assignments::end_date.is_null())
                .select(role_assignments::role_id)
                .first::<Uuid>(conn)
                .optional()?;
            if current_role != offer.from_role_id {
                return Ok(Err(
                    "The candidate has already moved; this offer is no longer valid".to_string(),
                ));
            }

            // Vacancy: the target role must still be active and unfilled.
            let target: Role = roles::table.filter(roles::id.eq(offer.role_id)).first(conn)?;
            if !target.active {
                return Ok(Err("The offered role is no longer active".to_string()));
            }
            if target.person_id.is_some() {
                return Ok(Err("The offered role has been filled in the meantime".to_string()));
            }

            // Execute the transfer in this same transaction.
            Role::assign_person_txn(conn, &offer.role_id, &offer.person_id)?;

            let updated: RoleOffer = diesel::update(role_offers::table.filter(role_offers::id.eq(offer_id)))
                .set((
                    role_offers::status.eq(RoleOfferStatus::Completed),
                    role_offers::decided_by_role_id.eq(decider_role_id),
                    role_offers::decided_at.eq(Some(now)),
                    role_offers::decision_note.eq(note),
                    role_offers::updated_at.eq(now),
                ))
                .get_result(conn)?;

            Ok(Ok(updated))
        })?;

        outcome.map_err(Error::new)
    }

    /// Decline a pending offer.
    pub fn decline(offer_id: &Uuid, decider_role_id: Option<Uuid>, note: Option<String>) -> Result<Self> {
        Self::close_pending(offer_id, RoleOfferStatus::Declined, decider_role_id, note)
    }

    /// Withdraw a pending offer (by the offerer).
    pub fn withdraw(offer_id: &Uuid, note: Option<String>) -> Result<Self> {
        Self::close_pending(offer_id, RoleOfferStatus::Withdrawn, None, note)
    }

    fn close_pending(
        offer_id: &Uuid,
        status: RoleOfferStatus,
        decider_role_id: Option<Uuid>,
        note: Option<String>,
    ) -> Result<Self> {
        let mut conn = connection()?;
        let offer = Self::get_by_id(offer_id)?;
        if offer.status != RoleOfferStatus::Pending {
            return Err(Error::new("This offer is no longer pending"));
        }
        let now = Utc::now().naive_utc();
        let updated = diesel::update(role_offers::table.filter(role_offers::id.eq(offer_id)))
            .set((
                role_offers::status.eq(status),
                role_offers::decided_by_role_id.eq(decider_role_id),
                role_offers::decided_at.eq(Some(now)),
                role_offers::decision_note.eq(note),
                role_offers::updated_at.eq(now),
            ))
            .get_result(&mut conn)?;
        Ok(updated)
    }
}
