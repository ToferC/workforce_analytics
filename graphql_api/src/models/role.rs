use std::{fmt::Debug, collections::HashMap};

use chrono::{prelude::*};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use diesel::{self, Insertable, Queryable, ExpressionMethods, Connection};
use rand::{distributions::{Distribution, Standard}, Rng};
use diesel::{RunQueryDsl, QueryDsl};
use uuid::Uuid;
use async_graphql::*;
use async_graphql::dataloader::DataLoader;

use crate::config_variables::DATE_FORMAT;
use crate::graphql::loaders::{EffortByRoleLoader, PersonLoader, RequirementsByRoleLoader, TeamLoader, WorkByRoleLoader};

use crate::schema::*;
use crate::database::{connection, DbConnection};

use super::{Person, Team, TeamOwnership, OrgTier, Work, Requirement, Capability, Product, RoleMatchResult, find_fuzzy_matches};

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = roles)]
#[diesel(belongs_to(Person))]
#[diesel(belongs_to(Team))]
/// Intermediary data structure between Person and team
/// Referenced by Person
pub struct Role {
    pub id: Uuid,
    pub person_id: Option<Uuid>, // You can have an empty role on a team
    pub team_id: Uuid,
    pub title_en: String,
    pub title_fr: String,
    pub effort: f64,
    pub active: bool,
    // HR info - this will be another module - just here for expediency
    // Military personnel use military_occupation and rank
    // Civilian personnel use occupational_group and occupational_level
    pub military_occupation: Option<MilitaryOccupation>,
    pub rank: Option<Rank>,
    pub occupational_group: Option<OccupationalGroup>,
    pub occupational_level: Option<i32>,

    pub start_datestamp: NaiveDateTime,
    pub end_date: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,

    /// The Role this position reports to. Position->position (not person->
    /// person): you report to a position, which may be vacant. NULL means
    /// "reports to my team's owner role" (see `manager`), so this is only set
    /// when a position needs a manager other than the default team owner.
    pub reports_to: Option<Uuid>,
}

#[Object]
impl Role {

    pub async fn id(&self) -> Uuid {
        self.id
    }

    pub async fn person(&self, ctx: &Context<'_>) -> Option<Person> {
        // Batched via DataLoader. Resolve to None on a missing row or load
        // error rather than panicking the worker for a transient DB hiccup.
        match self.person_id {
            Some(p) => ctx
                .data_unchecked::<DataLoader<PersonLoader>>()
                .load_one(p)
                .await
                .ok()
                .flatten(),
            None => None
        }
    }

    pub async fn team(&self, ctx: &Context<'_>) -> Result<Team> {
        ctx.data_unchecked::<DataLoader<TeamLoader>>()
            .load_one(self.team_id)
            .await?
            .ok_or_else(|| Error::new("Team not found"))
    }

    pub async fn title_english(&self) -> Result<String> {
        Ok(self.title_en.to_owned())
    }

    pub async fn title_french(&self) -> Result<String> {
        Ok(self.title_fr.to_owned())
    }

    /// Returns the sum effort of all active work underway
    /// Maximum work should be around 10
    pub async fn effort(&self, ctx: &Context<'_>) -> Result<i32> {
        // Batched via DataLoader into one grouped SUM per request; a role
        // with no active work has no entry, i.e. zero effort.
        Ok(ctx
            .data_unchecked::<DataLoader<EffortByRoleLoader>>()
            .load_one(self.id)
            .await?
            .unwrap_or(0))
    }

    /// Returns a vector of the work undertaken by this role
    pub async fn work(&self, ctx: &Context<'_>) -> Result<Vec<Work>> {
        Ok(ctx
            .data_unchecked::<DataLoader<WorkByRoleLoader>>()
            .load_one(self.id)
            .await?
            .unwrap_or_default())
    }

    pub async fn active(&self) -> Result<String> {
        if self.active {
            Ok("Active".to_string())
        } else {
            Ok("INACTIVE".to_string())
        }
    }

    pub async fn requirements(&self, ctx: &Context<'_>) -> Result<Vec<Requirement>> {
        Ok(ctx
            .data_unchecked::<DataLoader<RequirementsByRoleLoader>>()
            .load_one(self.id)
            .await?
            .unwrap_or_default())
    }

    /// Full tenure history for this position: who has occupied it and when,
    /// most recent first. The open assignment (no end_date) is the current
    /// occupant.
    pub async fn assignments(&self) -> Result<Vec<RoleAssignment>> {
        RoleAssignment::get_by_role_id(&self.id)
    }

    /// Returns the military occupation for a military role holder, if applicable
    pub async fn military_occupation(&self) -> Result<Option<String>> {
        Ok(self.military_occupation.map(|mo| mo.to_string()))
    }

    /// Returns the military rank for a military role holder, if applicable
    pub async fn rank(&self) -> Result<Option<Rank>> {
        Ok(self.rank)
    }

    /// Returns the occupational group for a civilian role holder, if applicable
    pub async fn occupational_group(&self) -> Result<Option<OccupationalGroup>> {
        Ok(self.occupational_group)
    }

    /// Returns the occupational level for a civilian role holder, if applicable
    pub async fn occupational_level(&self) -> Result<Option<i32>> {
        Ok(self.occupational_level)
    }

    pub async fn start_date(&self) -> Result<String> {
        Ok(self.start_datestamp.format(DATE_FORMAT).to_string())
    }

    pub async fn end_date(&self) -> Result<String> {
        match self.end_date {
            Some(d) => Ok(d.format(DATE_FORMAT).to_string()),
            None => Ok("Still Active".to_string())
        }
    }

    pub async fn created_at(&self) -> Result<String> {
        Ok(self.created_at.format(DATE_FORMAT).to_string())
    }

    pub async fn updated_at(&self) -> Result<String> {
        Ok(self.updated_at.format(DATE_FORMAT).to_string())
    }

    pub async fn find_matches(&self) -> Result<Vec<Person>> {

        let requirements = Requirement::get_by_role_id(self.id)?;

        find_people_by_requirements_met(requirements)
    }

    /// Returns tiered candidates for this role.
    ///
    /// `full_matches` meet every requirement at or above the required level.
    /// `partial_matches` meet at least `min_coverage` of requirements and have
    /// no single skill gap exceeding `max_gap_per_req` levels.
    ///
    /// Each capability level is a significant leap, so each missing level
    /// costs 10 points off the composite score (0–100 scale).
    pub async fn fuzzy_matches(
        &self,
        #[graphql(default = 0.5)] min_coverage: f64,
        #[graphql(default = 1)]   max_gap_per_req: i32,
        #[graphql(default = 20)]  limit: i32,
    ) -> Result<RoleMatchResult> {
        // Multi-query scoring pass; run off the async executor.
        let role_id = self.id;
        crate::graphql::loaders::off_executor(move || {
            find_fuzzy_matches(role_id, min_coverage, max_gap_per_req, limit as usize)
        })
        .await
    }

    pub async fn start_datestamp(&self) -> Result<NaiveDateTime> {
        Ok(self.start_datestamp)
    }

    pub async fn end_datestamp(&self) -> Result<Option<NaiveDateTime>> {
        Ok(self.end_date)
    }

    pub async fn owned_products(&self) -> Result<Vec<Product>> {
        let mut conn = connection()?;
        let product = products::table
            .filter(products::product_owner_role_id.eq(self.id))
            .load::<Product>(&mut conn)?;
        Ok(product)
    }

    /// Raw id of the position this role explicitly reports to, if any. NULL
    /// means it falls back to the team owner — use `manager` for the resolved
    /// effective manager.
    pub async fn reports_to_id(&self) -> Option<Uuid> {
        self.reports_to
    }

    /// The position this role explicitly reports to, if one is set. Returns
    /// None when the role inherits its team owner as manager (see `manager`).
    pub async fn reports_to(&self) -> Result<Option<Role>> {
        match self.reports_to {
            Some(id) => Ok(Some(Role::get_by_id(&id)?)),
            None => Ok(None),
        }
    }

    /// The effective manager position for this role: its explicit `reports_to`
    /// if set, otherwise the owner role of its team. Returns None only when the
    /// role is itself the team owner (or the team has no owner), i.e. there is
    /// no position above it within the team. This is the edge the org chart
    /// should draw.
    pub async fn manager(&self) -> Result<Option<Role>> {
        if let Some(id) = self.reports_to {
            return Ok(Some(Role::get_by_id(&id)?));
        }
        match TeamOwnership::get_by_team_id(&self.team_id) {
            Ok(ownership) if ownership.owner_role_id != self.id => {
                Ok(Some(Role::get_by_id(&ownership.owner_role_id)?))
            }
            _ => Ok(None),
        }
    }

    /// The positions that report directly to this one (explicit edges only).
    pub async fn direct_reports(&self) -> Result<Vec<Role>> {
        Role::get_direct_reports(&self.id)
    }

    /// A single seniority score for this position: the org-tier level dominates
    /// (a more senior tier outranks any rank below it) and rank/classification
    /// breaks ties within a tier. Higher = more senior. Comparable across the
    /// whole org when tiers differ, and within a single personnel stream
    /// (military or civilian) when tiers are equal; equal tier across different
    /// streams is not meaningfully comparable. Null when the tier is unknown.
    pub async fn seniority_score(&self) -> Result<Option<i64>> {
        let mut conn = connection()?;
        let s = seniority_of(&mut conn, &self.team_id, self.rank, self.occupational_level);
        Ok(s.score())
    }
}


// Non Graphql
impl Role {
    pub fn create(role: &NewRole) -> Result<Role> {
        let mut conn = connection()?;

        let res: Role = diesel::insert_into(roles::table)
        .values(role)
        .get_result(&mut conn)?;

        // If the role is created already occupied, open a tenure so the
        // assignment history reflects the incumbent from day one.
        if let Some(person_id) = res.person_id {
            RoleAssignment::open(&res.id, &person_id, res.start_datestamp)?;
        }

        Ok(res)
    }

    pub fn batch_create(roles: Vec<NewRole>) -> Result<usize> {
        let mut conn = connection()?;

        let res = diesel::insert_into(roles::table)
            .values(roles)
            .execute(&mut conn)?;
        
        Ok(res)
    }
    
    pub fn get_or_create(role: &NewRole) -> Result<Role> {
        let mut conn = connection()?;

        let res = roles::table
        .filter(roles::person_id.eq(&role.person_id))
        .distinct()
        .first(&mut conn);
        
        let role = match res {
            Ok(p) => p,
            Err(e) => {
                // Role not found
                println!("{:?}", e);
                let p = Role::create(role).expect("Unable to create role");
                p
            }
        };
        Ok(role)
    }

    pub fn get_all_active() -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let roles = roles::table
            .filter(roles::active.eq(true))
            .load::<Role>(&mut conn)?;
        Ok(roles)
    }

    pub fn get_active(count: i64) -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let roles = roles::table
            .filter(roles::active.eq(true))
            .limit(count)
            .load::<Role>(&mut conn)?;
        
        Ok(roles)
    }

    pub fn count() -> Result<i64> {
        let mut conn = connection()?;

        let res = roles::table
            .count()
            .get_result(&mut conn)?;

        Ok(res)
    }

    pub fn get_by_id(id: &Uuid) -> Result<Self> {
        let mut conn = connection()?;
        let role = roles::table.filter(roles::id.eq(id)).first(&mut conn)?;
        Ok(role)
    }

    /// Batched lookup for the DataLoader: fetch many roles in one query.
    pub fn get_by_ids(ids: &[Uuid]) -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let res = roles::table.filter(roles::id.eq_any(ids)).load::<Role>(&mut conn)?;
        Ok(res)
    }

    /// Positions that report directly to the given role (explicit edges only).
    pub fn get_direct_reports(id: &Uuid) -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let res = roles::table
            .filter(roles::reports_to.eq(id))
            .load::<Role>(&mut conn)?;
        Ok(res)
    }

    /// Set (or clear, with None) the position this role reports to. Rejects a
    /// self-reference and any edge that would introduce a cycle: the reporting
    /// graph must stay acyclic so the org chart is a tree.
    pub fn set_reports_to(role_id: &Uuid, manager: Option<Uuid>) -> Result<Self> {
        let mut conn = connection()?;

        if let Some(manager_id) = manager {
            if manager_id == *role_id {
                return Err(Error::new("A role cannot report to itself"));
            }

            // Walk up from the proposed manager; if we reach role_id, the edge
            // would close a cycle. Load (id, reports_to) pairs once and follow
            // the chain in memory.
            let edges: Vec<(Uuid, Option<Uuid>)> = roles::table
                .select((roles::id, roles::reports_to))
                .load::<(Uuid, Option<Uuid>)>(&mut conn)?;

            let mut current = Some(manager_id);
            let mut seen = std::collections::HashSet::new();
            while let Some(node) = current {
                if node == *role_id {
                    return Err(Error::new(
                        "That reporting line would create a cycle",
                    ));
                }
                if !seen.insert(node) {
                    break; // pre-existing cycle in data; stop rather than loop
                }
                current = edges.iter().find(|(id, _)| *id == node).and_then(|(_, p)| *p);
            }

            // An organization is a hierarchy: a position must report to a
            // strictly more senior one. A definitive peer/junior line is
            // rejected; an indeterminate one (missing rank, or cross-stream
            // military↔civilian) is allowed so incomplete HR data can't block
            // legitimate structure.
            let role = Role::get_by_id(role_id)?;
            let mgr = Role::get_by_id(&manager_id)?;
            if seniority_validation_enabled()
                && compare_seniority(&mut conn, &role, &mgr) == SeniorityCmp::NotSenior
            {
                return Err(Error::new(
                    "A role must report to a more senior position — it cannot report to a peer of the same rank or to a more junior role",
                ));
            }
        }

        let now = chrono::Utc::now().naive_utc();
        let res = diesel::update(roles::table.filter(roles::id.eq(role_id)))
            .set((roles::reports_to.eq(manager), roles::updated_at.eq(now)))
            .get_result(&mut conn)?;

        Ok(res)
    }

    /// Validate a proposed reporting line at role-creation time, before the new
    /// role exists. Mirrors the seniority rule in `set_reports_to`. The new
    /// role's seniority is derived from the fields it will be created with.
    pub fn check_create_reports_to(
        team_id: &Uuid,
        rank: Option<Rank>,
        occupational_level: Option<i32>,
        manager_id: &Uuid,
    ) -> Result<()> {
        if !seniority_validation_enabled() {
            return Ok(());
        }
        let mut conn = connection()?;
        let mgr = Role::get_by_id(manager_id)?;
        let role_s = seniority_of(&mut conn, team_id, rank, occupational_level);
        let mgr_s = seniority_of(&mut conn, &mgr.team_id, mgr.rank, mgr.occupational_level);
        if compare(&role_s, &mgr_s) == SeniorityCmp::NotSenior {
            return Err(Error::new(
                "A role must report to a more senior position — it cannot report to a peer of the same rank or to a more junior role",
            ));
        }
        Ok(())
    }

    pub fn get_active_vacant_by_ids(ids: &Vec<Uuid>) -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let roles = roles::table
            .filter(roles::id.eq_any(ids))
            .filter(roles::active.eq(true))
            .filter(roles::person_id.is_null())
            .load::<Self>(&mut conn)?;
        Ok(roles)
    }

    pub fn get_by_team_id(id: Uuid) -> Result<Vec<Role>> {
        let mut conn = connection()?;

        let res = roles::table
            .filter(roles::team_id.eq(id))
            .load::<Role>(&mut conn)?;

        Ok(res)
    }

    /// Returns active and occupied roles by a team_id
    pub fn get_occupied_by_team_id(id: Uuid) -> Result<Vec<Role>> {
        let mut conn = connection()?;

        let res = roles::table
            .filter(roles::team_id.eq(id))
            .filter(roles::active.eq(true))
            .filter(roles::person_id.is_not_null())
            .load::<Role>(&mut conn)?;

        Ok(res)
    }

    /// Returns vacant and active roles
    pub fn get_vacant(count: i64) -> Result<Vec<Role>> {
        let mut conn = connection()?;

        let res = roles::table
            .filter(roles::person_id.is_null())
            .filter(roles::active.eq(true))
            .limit(count)
            .load::<Role>(&mut conn)?;

        Ok(res)
    }

    /// Returns vacant and active roles by a team_id
    pub fn get_vacant_by_team_id(id: Uuid) -> Result<Vec<Role>> {
        let mut conn = connection()?;

        let res = roles::table
            .filter(roles::team_id.eq(id))
            .filter(roles::person_id.is_null())
            .filter(roles::active.eq(true))
            .load::<Role>(&mut conn)?;

        Ok(res)
    }

    /// Get roles by person ID. Can add a boolean to choose between active or inactive roles.
    pub fn get_by_person_id(id: Uuid, active: bool) -> Result<Vec<Role>> {
        let mut conn = connection()?;

        let res = roles::table
            .filter(roles::person_id.eq(id))
            .filter(roles::active.eq(active))
            .load::<Role>(&mut conn)?;

        Ok(res)
    }

    /// Roles this person currently occupies, derived from their open tenure.
    /// A person holds at most one active role at a time.
    pub fn get_current_for_person(person_id: &Uuid) -> Result<Vec<Role>> {
        let mut conn = connection()?;

        let role_ids: Vec<Uuid> = role_assignments::table
            .filter(role_assignments::person_id.eq(person_id))
            .filter(role_assignments::end_date.is_null())
            .select(role_assignments::role_id)
            .load::<Uuid>(&mut conn)?;

        let roles = roles::table
            .filter(roles::id.eq_any(&role_ids))
            .load::<Role>(&mut conn)?;

        Ok(roles)
    }

    /// Roles this person used to occupy, derived from their closed tenures.
    /// This is the person's career progression through positions and survives
    /// reassignment of the position to someone else.
    pub fn get_past_for_person(person_id: &Uuid) -> Result<Vec<Role>> {
        let mut conn = connection()?;

        let role_ids: Vec<Uuid> = role_assignments::table
            .filter(role_assignments::person_id.eq(person_id))
            .filter(role_assignments::end_date.is_not_null())
            .select(role_assignments::role_id)
            .load::<Uuid>(&mut conn)?;

        let roles = roles::table
            .filter(roles::id.eq_any(&role_ids))
            .load::<Role>(&mut conn)?;

        Ok(roles)
    }
    
    pub fn update(&mut self) -> Result<Self> {
        let mut conn = connection()?;

        self.updated_at = chrono::Utc::now().naive_utc();

        let res = diesel::update(roles::table)
        .filter(roles::id.eq(&self.id))
        .set(self.clone())
        .get_result(&mut conn)?;

        Ok(res)
    }

    /// Assign a person to this role. The role itself is a durable position, so
    /// assigning rotates occupants rather than failing:
    ///   * any current occupant of this role has their tenure closed (vacated);
    ///   * the person's open tenure on any other role is closed, enforcing one
    ///     active role per person and recording the move as career history;
    ///   * a fresh open tenure is started and the role's current-occupant
    ///     pointer is updated.
    /// All of this happens in a single transaction so history stays consistent.
    pub fn assign_person(role_id: &Uuid, person_id: &Uuid) -> Result<Self> {
        let mut conn = connection()?;

        let role = conn.transaction::<Role, diesel::result::Error, _>(|conn| {
            Role::assign_person_txn(conn, role_id, person_id)
        })?;

        Ok(role)
    }

    /// The body of [`assign_person`], runnable inside a caller's transaction so
    /// the assignment commits atomically with surrounding work (e.g. accepting
    /// a transfer offer marks the offer Completed in the same transaction).
    pub fn assign_person_txn(
        conn: &mut DbConnection,
        role_id: &Uuid,
        person_id: &Uuid,
    ) -> std::result::Result<Self, diesel::result::Error> {
        let now = chrono::Utc::now().naive_utc();

        // Close the current occupant's tenure on this role, if any.
        diesel::update(
            role_assignments::table
                .filter(role_assignments::role_id.eq(role_id))
                .filter(role_assignments::end_date.is_null()),
        )
        .set((
            role_assignments::end_date.eq(now),
            role_assignments::updated_at.eq(now),
        ))
        .execute(conn)?;

        // Close this person's open tenure on any other role.
        diesel::update(
            role_assignments::table
                .filter(role_assignments::person_id.eq(person_id))
                .filter(role_assignments::end_date.is_null()),
        )
        .set((
            role_assignments::end_date.eq(now),
            role_assignments::updated_at.eq(now),
        ))
        .execute(conn)?;

        // Clear the current-occupant pointer on the role(s) the person just
        // left, so those positions read as vacant.
        diesel::update(
            roles::table
                .filter(roles::person_id.eq(person_id))
                .filter(roles::id.ne(role_id)),
        )
        .set((
            roles::person_id.eq(None::<Uuid>),
            roles::updated_at.eq(now),
        ))
        .execute(conn)?;

        // Open the new tenure.
        diesel::insert_into(role_assignments::table)
            .values(NewRoleAssignment {
                role_id: *role_id,
                person_id: *person_id,
                start_date: now,
                end_date: None,
            })
            .execute(conn)?;

        // Point the role at its new current occupant.
        let updated = diesel::update(roles::table.filter(roles::id.eq(role_id)))
            .set((
                roles::person_id.eq(Some(person_id)),
                roles::updated_at.eq(now),
            ))
            .get_result(conn)?;

        Ok(updated)
    }

    /// Remove the person from this role, leaving the position vacant but still
    /// active. Closes the open tenure (recording it as career history) and
    /// clears the current-occupant pointer. The role itself is untouched.
    pub fn vacate(role_id: &Uuid) -> Result<Self> {
        let mut conn = connection()?;

        let role = conn.transaction::<Role, diesel::result::Error, _>(|conn| {
            let now = chrono::Utc::now().naive_utc();

            diesel::update(
                role_assignments::table
                    .filter(role_assignments::role_id.eq(role_id))
                    .filter(role_assignments::end_date.is_null()),
            )
            .set((
                role_assignments::end_date.eq(now),
                role_assignments::updated_at.eq(now),
            ))
            .execute(conn)?;

            let updated = diesel::update(roles::table.filter(roles::id.eq(role_id)))
                .set((
                    roles::person_id.eq(None::<Uuid>),
                    roles::updated_at.eq(now),
                ))
                .get_result(conn)?;

            Ok(updated)
        })?;

        Ok(role)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Insertable, InputObject)]
#[diesel(table_name = roles)]
pub struct NewRole {
    pub person_id: Option<Uuid>,
    pub team_id: Uuid,
    pub title_en: String,
    pub title_fr: String,
    pub effort: f64,
    pub active: bool,
    // HR info - this will be another module - just here for expediency
    // Military personnel use military_occupation and rank
    // Civilian personnel use occupational_group and occupational_level
    pub military_occupation: Option<MilitaryOccupation>,
    pub rank: Option<Rank>,
    pub occupational_group: Option<OccupationalGroup>,
    pub occupational_level: Option<i32>,
    pub start_datestamp: NaiveDateTime,
    pub end_date: Option<NaiveDateTime>,
    /// Optional reporting line to another Role. Defaults to None (falls back to
    /// the team owner). See `Role::reports_to`.
    #[graphql(default)]
    pub reports_to: Option<Uuid>,
}

impl NewRole {

    pub fn new(
        person_id: Option<Uuid>,
        team_id: Uuid,
        title_en: String,
        title_fr: String,
        effort: f64,
        active: bool,
        military_occupation: Option<MilitaryOccupation>,
        rank: Option<Rank>,
        occupational_group: Option<OccupationalGroup>,
        occupational_level: Option<i32>,
        start_datestamp: NaiveDateTime,
        end_date: Option<NaiveDateTime>,
    ) -> Self {
        NewRole {
            person_id,
            team_id,
            title_en,
            title_fr,
            effort,
            active,
            military_occupation,
            rank,
            occupational_group,
            occupational_level,
            start_datestamp,
            end_date,
            reports_to: None,
        }
    }
}

/// Records a single person's tenure in a Role. The Role is durable; the
/// assignment captures *who* held it and *when*. An open assignment
/// (`end_date` is None) is the current occupant; closed assignments form the
/// person's career history.
#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Identifiable, Insertable, AsChangeset)]
#[diesel(table_name = role_assignments)]
pub struct RoleAssignment {
    pub id: Uuid,
    pub role_id: Uuid,
    pub person_id: Uuid,
    pub start_date: NaiveDateTime,
    pub end_date: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[Object]
impl RoleAssignment {
    pub async fn id(&self) -> Uuid {
        self.id
    }

    /// The person who held (or holds) the role during this tenure.
    pub async fn person(&self) -> Result<Person> {
        Person::get_by_id(&self.person_id)
    }

    /// The durable position this tenure belongs to.
    pub async fn role(&self) -> Result<Role> {
        Role::get_by_id(&self.role_id)
    }

    pub async fn start_date(&self) -> Result<String> {
        Ok(self.start_date.format(DATE_FORMAT).to_string())
    }

    pub async fn end_date(&self) -> Result<String> {
        match self.end_date {
            Some(d) => Ok(d.format(DATE_FORMAT).to_string()),
            None => Ok("Current".to_string()),
        }
    }

    /// True while the person still occupies the role (no end_date set).
    pub async fn is_current(&self) -> bool {
        self.end_date.is_none()
    }

    pub async fn start_datestamp(&self) -> NaiveDateTime {
        self.start_date
    }

    pub async fn end_datestamp(&self) -> Option<NaiveDateTime> {
        self.end_date
    }
}

impl RoleAssignment {
    /// Open a new tenure for a person in a role (end_date None).
    pub fn open(role_id: &Uuid, person_id: &Uuid, start: NaiveDateTime) -> Result<Self> {
        let mut conn = connection()?;

        let res = diesel::insert_into(role_assignments::table)
            .values(NewRoleAssignment {
                role_id: *role_id,
                person_id: *person_id,
                start_date: start,
                end_date: None,
            })
            .get_result(&mut conn)?;

        Ok(res)
    }

    /// Close the open tenure on a role at a given date and clear the role's
    /// current-occupant pointer. Used to seed career history.
    pub fn close_open_for_role(role_id: &Uuid, end: NaiveDateTime) -> Result<()> {
        let mut conn = connection()?;

        diesel::update(
            role_assignments::table
                .filter(role_assignments::role_id.eq(role_id))
                .filter(role_assignments::end_date.is_null()),
        )
        .set((
            role_assignments::end_date.eq(end),
            role_assignments::updated_at.eq(end),
        ))
        .execute(&mut conn)?;

        diesel::update(roles::table.filter(roles::id.eq(role_id)))
            .set(roles::person_id.eq(None::<Uuid>))
            .execute(&mut conn)?;

        Ok(())
    }

    /// Enforce one active role per person: close the person's open tenures on
    /// every role except `keep_role_id`, and clear those roles' current-occupant
    /// pointers so they read as vacant. Recorded as career history.
    pub fn close_others_for_person(person_id: &Uuid, keep_role_id: &Uuid) -> Result<()> {
        let mut conn = connection()?;
        let now = chrono::Utc::now().naive_utc();

        diesel::update(
            role_assignments::table
                .filter(role_assignments::person_id.eq(person_id))
                .filter(role_assignments::end_date.is_null())
                .filter(role_assignments::role_id.ne(keep_role_id)),
        )
        .set((
            role_assignments::end_date.eq(now),
            role_assignments::updated_at.eq(now),
        ))
        .execute(&mut conn)?;

        diesel::update(
            roles::table
                .filter(roles::person_id.eq(person_id))
                .filter(roles::id.ne(keep_role_id)),
        )
        .set((roles::person_id.eq(None::<Uuid>), roles::updated_at.eq(now)))
        .execute(&mut conn)?;

        Ok(())
    }

    /// All tenures for a position, most recent first.
    pub fn get_by_role_id(role_id: &Uuid) -> Result<Vec<Self>> {
        let mut conn = connection()?;

        let res = role_assignments::table
            .filter(role_assignments::role_id.eq(role_id))
            .order(role_assignments::start_date.desc())
            .load::<Self>(&mut conn)?;

        Ok(res)
    }

    /// All tenures for a person, most recent first — their career history.
    pub fn get_by_person_id(person_id: &Uuid) -> Result<Vec<Self>> {
        let mut conn = connection()?;

        let res = role_assignments::table
            .filter(role_assignments::person_id.eq(person_id))
            .order(role_assignments::start_date.desc())
            .load::<Self>(&mut conn)?;

        Ok(res)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Insertable)]
#[diesel(table_name = role_assignments)]
pub struct NewRoleAssignment {
    pub role_id: Uuid,
    pub person_id: Uuid,
    pub start_date: NaiveDateTime,
    pub end_date: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Enum, DbEnum, Copy, Display)]
#[ExistingTypePath = "crate::schema::sql_types::Rank"]
/// Represents military rank structure
pub enum Rank {
    Private,
    Corporal,
    MasterCorporal,
    Sergeant,
    WarrantOfficer,
    MasterWarrantOfficer,
    ChiefWarrantOfficer,
    SecondLieutenant,
    Lieutenant,
    Captain,
    Major,
    LieutenantColonel,
    Colonel,
    BrigadierGeneral,
    MajorGeneral,
    LieutenantGeneral,
    General,
}

impl Rank {
    /// Returns the next rank (promotion)
    /// Returns the same rank if already at the highest rank
    pub fn next(&self) -> Rank {
        match self {
            // Non-Commissioned Member progression
            Rank::Private => Rank::Corporal,
            Rank::Corporal => Rank::MasterCorporal,
            Rank::MasterCorporal => Rank::Sergeant,
            Rank::Sergeant => Rank::WarrantOfficer,
            Rank::WarrantOfficer => Rank::MasterWarrantOfficer,
            Rank::MasterWarrantOfficer => Rank::ChiefWarrantOfficer,
            Rank::ChiefWarrantOfficer => Rank::Lieutenant, // Highest NCM rank
            
            // Officer progression
            Rank::SecondLieutenant => Rank::Lieutenant,
            Rank::Lieutenant => Rank::Captain,
            Rank::Captain => Rank::Major,
            Rank::Major => Rank::LieutenantColonel,
            Rank::LieutenantColonel => Rank::Colonel,
            Rank::Colonel => Rank::BrigadierGeneral,
            Rank::BrigadierGeneral => Rank::MajorGeneral,
            Rank::MajorGeneral => Rank::LieutenantGeneral,
            Rank::LieutenantGeneral => Rank::General,
            Rank::General => Rank::General, // Highest rank
        }
    }

    /// Returns the previous rank (demotion)
    /// Returns the same rank if already at the lowest rank
    pub fn previous(&self) -> Rank {
        match self {
            // Non-Commissioned Member regression
            Rank::Private => Rank::Private, // Lowest rank
            Rank::Corporal => Rank::Private,
            Rank::MasterCorporal => Rank::Corporal,
            Rank::Sergeant => Rank::MasterCorporal,
            Rank::WarrantOfficer => Rank::Sergeant,
            Rank::MasterWarrantOfficer => Rank::WarrantOfficer,
            Rank::ChiefWarrantOfficer => Rank::MasterWarrantOfficer,
            
            // Officer regression
            Rank::SecondLieutenant => Rank::SecondLieutenant, // Lowest officer rank
            Rank::Lieutenant => Rank::SecondLieutenant,
            Rank::Captain => Rank::Lieutenant,
            Rank::Major => Rank::Captain,
            Rank::LieutenantColonel => Rank::Major,
            Rank::Colonel => Rank::LieutenantColonel,
            Rank::BrigadierGeneral => Rank::Colonel,
            Rank::MajorGeneral => Rank::BrigadierGeneral,
            Rank::LieutenantGeneral => Rank::MajorGeneral,
            Rank::General => Rank::LieutenantGeneral,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Enum, DbEnum, Copy, Display)]
#[ExistingTypePath = "crate::schema::sql_types::MilitaryOccupation"]
/// Represents Canadian Army military occupations
pub enum MilitaryOccupation {
    Infantry,
    Armoured,
    Artillery,
    CombatEngineers,
    Signals,
    Intelligence,
    MilitaryPolice,
    LogisticsSupport,
    MedicalTechnician,
    Communications,
    Electronics,
    VehicleTechnician,
    WeaponsTechnician,
    SupplyTechnician,
    CookSupport,
    FinanceClerk,
    HumanResourcesAdministrator,
    MilitaryFirefighter,
    MaterialsManagement,
    GeomaticsTechnician,
    MedicalAssistant,
    DentalAssistant,
    PharmacyTechnician,
    Chaplain,
    LegalOfficer,
    Pilot,
    AircrewSystems,
    AirTrafficController,
    WeatherTechnician,
    ImageTechnician,
    Musician,
    PhysicalFitnessInstructor,
    Cyber,
    SpecialForces,
    Officer,
    Other,
}

impl MilitaryOccupation {
    pub fn choose() -> MilitaryOccupation {
        let choice: MilitaryOccupation = rand::random();
        choice
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Enum, DbEnum, Copy, Display)]
#[ExistingTypePath = "crate::schema::sql_types::OccupationalGroup"]
/// Represents civilian occupational groups (classifications)
pub enum OccupationalGroup {
    AdministrativeServices,
    ComputerSystems,
    EconomicsAndSocialScience,
    Engineering,
    Executive,
    FinancialManagement,
    HumanResources,
    InformationServices,
    ProgramAdministration,
    Research,
    TechnicalServices,
    Other,
}

impl OccupationalGroup {
    pub fn choose() -> OccupationalGroup {
        let choice: OccupationalGroup = rand::random();
        choice
    }
}

impl Distribution<OccupationalGroup> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> OccupationalGroup {
        match rng.gen_range(0..100) {
            0..=19 => OccupationalGroup::AdministrativeServices,    // 20% - Administration
            20..=34 => OccupationalGroup::ProgramAdministration,    // 15% - Program delivery
            35..=46 => OccupationalGroup::ComputerSystems,          // 12% - IT
            47..=56 => OccupationalGroup::EconomicsAndSocialScience, // 10% - Analysis
            57..=64 => OccupationalGroup::TechnicalServices,        // 8% - Technical
            65..=72 => OccupationalGroup::Engineering,              // 8% - Engineering
            73..=79 => OccupationalGroup::FinancialManagement,      // 7% - Finance
            80..=86 => OccupationalGroup::HumanResources,           // 7% - HR
            87..=91 => OccupationalGroup::Research,                 // 5% - Research
            92..=95 => OccupationalGroup::InformationServices,      // 4% - Communications
            96..=97 => OccupationalGroup::Executive,                // 2% - Executives
            _ => OccupationalGroup::Other,                          // 2% - Miscellaneous
        }
    }
}

impl Distribution<Rank> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Rank {
        match rng.gen_range(0..100) {
            0..=24 => Rank::Private,          // 25% - Entry level
            25..=44 => Rank::Corporal,        // 20% - Junior NCO
            45..=59 => Rank::MasterCorporal,  // 15% - Senior NCO
            60..=74 => Rank::Sergeant,        // 15% - Staff NCO
            75..=84 => Rank::WarrantOfficer,  // 10% - Warrant Officers
            85..=89 => Rank::MasterWarrantOfficer,     // 5%
            90..=92 => Rank::ChiefWarrantOfficer,      // 3%
            93..=95 => Rank::SecondLieutenant,         // 3% - Junior Officers
            96..=97 => Rank::Lieutenant,               // 2%
            98 => Rank::Captain,                       // 1%
            99 => Rank::Major,                         // 1% - Senior Officers
            100 => Rank::LieutenantColonel,            // <1%
            101 => Rank::Colonel,                      // <1%
            102 => Rank::BrigadierGeneral,             // <1% - Flag Officers
            103 => Rank::MajorGeneral,                 // <1%
            104 => Rank::LieutenantGeneral,            // <1%
            _ => Rank::General,                        // <1%
        }
    }
}

impl Distribution<MilitaryOccupation> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> MilitaryOccupation {
        match rng.gen_range(0..100) {
            0..=14 => MilitaryOccupation::Infantry,           // 15% - Core combat role
            15..=24 => MilitaryOccupation::LogisticsSupport,  // 10% - Essential support
            25..=32 => MilitaryOccupation::Communications,    // 8% - Critical infrastructure
            33..=39 => MilitaryOccupation::VehicleTechnician, // 7% - Maintenance
            40..=46 => MilitaryOccupation::SupplyTechnician,  // 7% - Supply chain
            47..=52 => MilitaryOccupation::MedicalTechnician, // 6% - Healthcare
            53..=58 => MilitaryOccupation::Artillery,         // 6% - Fire support
            59..=63 => MilitaryOccupation::CombatEngineers,   // 5% - Engineering
            64..=68 => MilitaryOccupation::Signals,           // 5% - Communications
            69..=72 => MilitaryOccupation::Armoured,          // 4% - Armoured corps
            73..=76 => MilitaryOccupation::Intelligence,      // 4% - Intel gathering
            77..=80 => MilitaryOccupation::Electronics,       // 4% - Technical support
            81..=83 => MilitaryOccupation::WeaponsTechnician, // 3% - Weapons maintenance
            84..=86 => MilitaryOccupation::MilitaryPolice,    // 3% - Security
            87..=89 => MilitaryOccupation::FinanceClerk,      // 3% - Administration
            90..=91 => MilitaryOccupation::HumanResourcesAdministrator, // 2%
            92 => MilitaryOccupation::CookSupport,       // 1% - Food services
            93 => MilitaryOccupation::Officer,          // 1% - Officer
            94 => MilitaryOccupation::MedicalAssistant,       // 1%
            95 => MilitaryOccupation::GeomaticsTechnician,    // 1%
            96 => MilitaryOccupation::MilitaryFirefighter,    // 1%
            97 => MilitaryOccupation::Cyber,                  // 1% - Emerging field
            98 => MilitaryOccupation::Pilot,                  // 1% - Specialized
            99 => MilitaryOccupation::SpecialForces,          // 1% - Elite units
            _ => MilitaryOccupation::Other,                   // <1% - Miscellaneous
        }
    }
}

pub fn find_people_by_requirements_met(requirements: Vec<Requirement>) -> Result<Vec<Person>> {

    let mut people_ids = Vec::new();

    let num_matches_required = *&requirements.len() as i32;

    for req in requirements {

        let caps = Capability::get_by_skill_id_and_level(req.skill_id, req.required_level)?;

        for c in caps {
            people_ids.push(c.person_id);
        };
    }

    let id_counts: HashMap<Uuid, i32> =
        people_ids.iter()
            .fold(HashMap::new(), |mut map, id| {
                *map.entry(*id).or_insert(0) += 1;
                map
            });

    let mut validated_ids: Vec<Uuid> = Vec::new();

    for (k, v) in id_counts {
        if v >= num_matches_required {
            validated_ids.push(k);
        }
    };


    Person::get_by_ids(&validated_ids)
}

// ── Seniority (reports_to hierarchy validation) ─────────────────────────────
//
// An organization is a hierarchy: a position must report to a strictly more
// senior one. The genuinely comparable axis across personnel streams is the
// org-tier *level* (L0 DM/CDS … L4 manager), which is stream-agnostic; within a
// single tier we break ties on military rank or civilian classification level.
// Comparing a military rank to a civilian classification is not well-defined, so
// equal-tier cross-stream (and any case with missing data) is treated as
// indeterminate — allowed, not blocked, so incomplete HR data can't wedge the
// org chart.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stream {
    Military,
    Civilian,
    Unknown,
}

struct Seniority {
    /// Org-tier level; lower is more senior. None when it can't be resolved.
    tier: Option<i64>,
    stream: Stream,
    /// Within-tier ordinal: military rank index or civilian level. Higher is
    /// more senior.
    index: Option<i64>,
}

impl Seniority {
    /// A single comparable score (higher = more senior): tier dominates,
    /// within-tier ordinal breaks ties. None when the tier is unknown.
    fn score(&self) -> Option<i64> {
        self.tier.map(|t| (100 - t) * 1000 + self.index.unwrap_or(0))
    }
}

#[derive(Debug, PartialEq, Eq)]
enum SeniorityCmp {
    /// The manager is definitively more senior — the edge is valid.
    ManagerSenior,
    /// The manager is definitively a peer (same rank) or junior — invalid.
    NotSenior,
    /// Seniority can't be compared (missing data or equal-tier cross-stream).
    Indeterminate,
}

/// Resolve a position's seniority from its team's org tier and HR fields.
fn seniority_of(
    conn: &mut DbConnection,
    team_id: &Uuid,
    rank: Option<Rank>,
    occupational_level: Option<i32>,
) -> Seniority {
    let tier = teams::table
        .find(*team_id)
        .first::<Team>(conn)
        .ok()
        .and_then(|t| org_tiers::table.find(t.org_tier_id).first::<OrgTier>(conn).ok())
        .map(|ot| ot.tier_level as i64);

    let stream = if rank.is_some() {
        Stream::Military
    } else if occupational_level.is_some() {
        Stream::Civilian
    } else {
        Stream::Unknown
    };

    let index = match stream {
        // Rank is a fieldless enum declared from junior to senior, so its
        // discriminant is the seniority ordinal.
        Stream::Military => rank.map(|r| r as i64),
        Stream::Civilian => occupational_level.map(|l| l as i64),
        Stream::Unknown => None,
    };

    Seniority { tier, stream, index }
}

/// Compare a role against a proposed manager.
fn compare(role: &Seniority, mgr: &Seniority) -> SeniorityCmp {
    match (role.tier, mgr.tier) {
        (Some(r), Some(m)) => {
            if m < r {
                return SeniorityCmp::ManagerSenior; // more senior tier
            }
            if m > r {
                return SeniorityCmp::NotSenior; // junior tier
            }
            // Same tier: only comparable within one personnel stream.
            if role.stream == Stream::Unknown
                || mgr.stream == Stream::Unknown
                || role.stream != mgr.stream
            {
                return SeniorityCmp::Indeterminate;
            }
            match (role.index, mgr.index) {
                (Some(ri), Some(mi)) if mi > ri => SeniorityCmp::ManagerSenior,
                (Some(ri), Some(mi)) if mi < ri => SeniorityCmp::NotSenior,
                (Some(_), Some(_)) => SeniorityCmp::NotSenior, // equal rank = peer
                _ => SeniorityCmp::Indeterminate,
            }
        }
        _ => SeniorityCmp::Indeterminate,
    }
}

fn compare_seniority(conn: &mut DbConnection, role: &Role, mgr: &Role) -> SeniorityCmp {
    let rs = seniority_of(conn, &role.team_id, role.rank, role.occupational_level);
    let ms = seniority_of(conn, &mgr.team_id, mgr.rank, mgr.occupational_level);
    compare(&rs, &ms)
}

/// Whether the seniority rule is enforced. Enabled by default; set
/// `DISABLE_SENIORITY_VALIDATION=true` to relax it during a grandfather window
/// while rank/classification data is being populated.
fn seniority_validation_enabled() -> bool {
    !matches!(
        std::env::var("DISABLE_SENIORITY_VALIDATION").ok().as_deref(),
        Some("true") | Some("1")
    )
}
