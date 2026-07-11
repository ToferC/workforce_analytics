use std::fmt::Debug;

use chrono::{prelude::*};
use serde::{Deserialize, Serialize};
use diesel::{self, Insertable, Queryable, BoolExpressionMethods, ExpressionMethods, PgTextExpressionMethods};
use diesel::{RunQueryDsl, QueryDsl};
use uuid::Uuid;
use async_graphql::*;
use async_graphql::dataloader::DataLoader;
use crate::models::{Organization, OrgTier};

use crate::config_variables::DATE_FORMAT;

use crate::schema::*;
use crate::database::connection;
use crate::graphql::loaders::RoleLoader;

use super::{Role, RoleAssignment, TeamOwnership, SkillDomain};
use super::{Contract, FinancialSummary, PayRate, contracts_summary, salary_summary};


#[derive(Debug, Clone, Deserialize, Serialize, Identifiable, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = teams)]
#[diesel(belongs_to(Organization))]
#[diesel(belongs_to(OrgTier))]
/// Referenced by Role
pub struct Team {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub org_tier_id: Uuid,
    pub primary_domain: SkillDomain,

    pub name_en: String,
    pub name_fr: String,

    pub description_en: String,
    pub description_fr: String,

    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub retired_at: Option<NaiveDateTime>,

    // pub milestones: Uuid // Refers to Github Milestones
}

// Non Graphql
impl Team {
    pub fn create(team: &NewTeam) -> Result<Team> {
        let mut conn = connection()?;

        let res = diesel::insert_into(teams::table)
        .values(team)
        .get_result(&mut conn)?;
        
        Ok(res)
    }
    
    pub fn get_or_create(team: &NewTeam) -> Result<Team> {
        let mut conn = connection()?;

        let res = teams::table
        .filter(teams::name_en.eq(&team.name_en))
        .filter(teams::name_fr.eq(&team.name_fr))
        .filter(teams::organization_id.eq(&team.organization_id))
        .distinct()
        .first(&mut conn);
        
        let team = match res {
            Ok(p) => p,
            Err(e) => {
                // Team not found
                if e.to_string() == "NotFound" {
                    println!("{:?}", e);
                }
                let p = Team::create(team).expect("Unable to create team");
                p
            }
        };
        Ok(team)
    }

    pub fn get_by_id(id: &Uuid) -> Result<Self> {
        let mut conn = connection()?;

        let res = teams::table
            .filter(teams::id.eq(id))
            .first(&mut conn)?;

        Ok(res)
    }

    pub fn get_all() -> Result<Vec<Self>> {
        let mut conn = connection()?;

        let res = teams::table.load::<Team>(&mut conn)?;
        Ok(res)
    }

    pub fn get_by_name(name: String) -> Result<Vec<Self>> {
        let mut conn = connection()?;

        let res = teams::table
            .filter(teams::name_en.ilike(format!("%{}%", name)).or(teams::name_fr.ilike(format!("%{}%", name))))
            .load::<Team>(&mut conn)?;

        Ok(res)
    }

    /// Server-side filtered + paginated team list. `search` matches
    /// name_en/name_fr (case-insensitive); retired teams are excluded unless
    /// `include_retired`. A `None` limit returns every matching row (preserving
    /// the old "fetch all" behaviour for callers that don't paginate).
    pub fn get_filtered(search: Option<&str>, include_retired: bool, limit: Option<i64>, offset: i64) -> Result<Vec<Self>> {
        let mut conn = connection()?;

        let mut query = teams::table.into_boxed();
        if !include_retired {
            query = query.filter(teams::retired_at.is_null());
        }
        if let Some(s) = search {
            let pattern = format!("%{}%", s);
            query = query.filter(teams::name_en.ilike(pattern.clone()).or(teams::name_fr.ilike(pattern)));
        }
        query = query.order_by(teams::name_en);
        if let Some(l) = limit {
            query = query.limit(l).offset(offset);
        }

        let res = query.load::<Team>(&mut conn)?;
        Ok(res)
    }

    /// Total number of teams matching the same filters as `get_filtered`,
    /// ignoring limit/offset — for driving pagination controls.
    pub fn count_filtered(search: Option<&str>, include_retired: bool) -> Result<i64> {
        let mut conn = connection()?;

        let mut query = teams::table.into_boxed();
        if !include_retired {
            query = query.filter(teams::retired_at.is_null());
        }
        if let Some(s) = search {
            let pattern = format!("%{}%", s);
            query = query.filter(teams::name_en.ilike(pattern.clone()).or(teams::name_fr.ilike(pattern)));
        }

        let total = query.count().get_result(&mut conn)?;
        Ok(total)
    }

    pub fn get_by_org_tier_id(id: &Uuid) -> Result<Vec<Self>> {
        let mut conn = connection()?;

        let res = teams::table
            .filter(teams::org_tier_id.eq(id))
            .load::<Team>(&mut conn)?;

        Ok(res)
    }

    pub fn get_by_ids(ids: &Vec<Uuid>) -> Result<Vec<Self>> {

        let mut conn = connection()?;

        let res = teams::table
            .filter(teams::id.eq_any(ids))
            .load::<Team>(&mut conn)?;

        Ok(res)
    }
    
    pub fn update(&self) -> Result<Self> {
        let mut conn = connection()?;

        let res = diesel::update(teams::table)
        .filter(teams::id.eq(&self.id))
        .set(self)
        .get_result(&mut conn)?;

        Ok(res)
    }

    /// Clear retired_at to un-retire (AsChangeset skips None, so set NULL).
    pub fn restore(id: &Uuid) -> Result<Self> {
        let mut conn = connection()?;

        let res = diesel::update(teams::table.filter(teams::id.eq(id)))
            .set(teams::retired_at.eq::<Option<NaiveDateTime>>(None))
            .get_result(&mut conn)?;

        Ok(res)
    }

    /// Resolve the id of the Role that owns (manages) this team: its own
    /// ownership record, else the owner inherited up the org tier chain.
    /// Non-async so it can be reused outside resolvers (e.g. match scoping).
    pub fn owner_role_id(&self) -> Result<Uuid> {
        match TeamOwnership::get_by_team_id(&self.id) {
            Ok(team_ownership) => Ok(team_ownership.owner_role_id),
            Err(_) => {
                let mut tier = OrgTier::get_by_id(&self.org_tier_id)?;
                loop {
                    match crate::models::OrgOwnership::get_by_org_tier_id(&tier.id) {
                        Ok(ownership) => return Ok(ownership.owner_role_id),
                        Err(_) => match tier.parent_tier {
                            Some(parent_id) => tier = OrgTier::get_by_id(&parent_id)?,
                            None => return Err(async_graphql::Error::new("No owner assigned to this team")),
                        },
                    }
                }
            },
        }
    }
}

#[Object]
impl Team {
    pub async fn id(&self) -> Uuid {
        self.id
    }

    pub async fn organization(&self) -> Result<Organization> {
        Organization::get_by_id(&self.organization_id)
    }

    pub async fn organization_level(&self) -> Result<OrgTier> {
        OrgTier::get_by_id(&self.org_tier_id)
    }

    pub async fn name_english(&self) -> Result<String> {
        Ok(self.name_en.to_owned())
    }

    pub async fn name_french(&self) -> Result<String> {
        Ok(self.name_en.to_owned())
    }

    pub async fn description_english(&self) -> Result<String> {
        Ok(self.name_en.to_owned())
    }

    pub async fn description_french(&self) -> Result<String> {
        Ok(self.name_en.to_owned())
    }

    pub async fn retired_at(&self) -> Result<String> {
        match self.retired_at {
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

    pub async fn occupied_roles(&self) -> Result<Vec<Role>> {
        Role::get_occupied_by_team_id(self.id)
    }

    pub async fn vacant_roles(&self) -> Result<Vec<Role>> {
        Role::get_vacant_by_team_id(self.id)
    }

    pub async fn roles(&self) -> Result<Vec<Role>> {
        Role::get_by_team_id(self.id)
    }

    /// Fiscal-year cost picture for the whole team: salary budget, projection
    /// and vacancy lapse across every role (priced by override or pay rate),
    /// plus the FY share of contracts under tasks created by the team's roles.
    pub async fn finances(&self) -> Result<FinancialSummary> {
        let team_id = self.id;
        crate::graphql::loaders::off_executor(move || {
            let today = Utc::now().date_naive();
            let roles = Role::get_by_team_id(team_id)?;
            let rates = PayRate::get_effective(Utc::now().naive_utc())?;

            let role_ids: Vec<Uuid> = roles.iter().map(|r| r.id).collect();
            let mut by_role: std::collections::HashMap<Uuid, Vec<(NaiveDate, Option<NaiveDate>)>> =
                std::collections::HashMap::new();
            for a in RoleAssignment::get_by_role_ids(&role_ids)? {
                by_role
                    .entry(a.role_id)
                    .or_default()
                    .push((a.start_date.date(), a.end_date.map(|e| e.date())));
            }

            let mut total = FinancialSummary {
                fiscal_year: super::fiscal_year_label(today),
                ..Default::default()
            };
            for role in &roles {
                let rate = role.annual_salary_cents.or_else(|| {
                    PayRate::rate_from(&rates, role.occupational_group, role.occupational_level, role.rank)
                });
                if let Some(rate) = rate {
                    let window = (role.start_datestamp.date(), role.end_date.map(|e| e.date()));
                    let empty = Vec::new();
                    let periods = by_role.get(&role.id).unwrap_or(&empty);
                    total.add(&salary_summary(rate, window, periods, today));
                }
            }

            let contracts = Contract::get_by_team_id(&team_id)?;
            total.add(&contracts_summary(&contracts, today));

            Ok(total)
        })
        .await
    }

    /// The role that owns (manages) this team. Ownership is tied to the
    /// position, so the owner may be a vacant role; expose `owner.person` for
    /// the current incumbent. Teams created without an explicit ownership
    /// record fall back to their org tier's owner (which itself inherits up the
    /// tier chain). Never panic here: an unwrap would kill the worker for any
    /// team missing ownership.
    pub async fn owner(&self, ctx: &Context<'_>) -> Result<Role> {
        ctx.data_unchecked::<DataLoader<RoleLoader>>()
            .load_one(self.owner_role_id()?)
            .await?
            .ok_or_else(|| Error::new("Owner role not found"))
    }

    /// Capability counts for people holding roles in this team.
    pub async fn capability_counts(&self) -> Result<Vec<crate::models::CapabilityCount>> {
        use crate::schema::{roles, capabilities};
        use diesel::dsl::count;
        use diesel::prelude::*;
        let mut conn = connection()?;

        let person_ids: Vec<Uuid> = roles::table
            .filter(roles::team_id.eq(self.id))
            .filter(roles::active.eq(true))
            .filter(roles::person_id.is_not_null())
            .select(roles::person_id)
            .load::<Option<Uuid>>(&mut conn)?
            .into_iter()
            .flatten()
            .collect();

        let res: Vec<(String, SkillDomain, Option<crate::models::CapabilityLevel>, i64)> = capabilities::table
            .filter(capabilities::person_id.eq_any(&person_ids))
            .filter(capabilities::retired_at.is_null())
            .group_by((capabilities::name_en, capabilities::domain, capabilities::validated_level))
            .select((capabilities::name_en, capabilities::domain, capabilities::validated_level, count(capabilities::id)))
            .order_by((capabilities::name_en, capabilities::validated_level))
            .load::<(String, SkillDomain, Option<crate::models::CapabilityLevel>, i64)>(&mut conn)?;

        Ok(res.into_iter().map(crate::models::CapabilityCount::from).collect())
    }

    /// Capability depth per skill domain for this team (lightweight heatmap row).
    pub async fn capability_heatmap(&self) -> Result<Vec<crate::models::TeamCapabilityCell>> {
        let rows = crate::graphql::query::compute_team_capability_matrix(None, Some(self.id))?;
        Ok(rows.into_iter().next().map(|r| r.cells).unwrap_or_default())
    }

    /// Number of distinct people holding active roles on this team.
    #[allow(deprecated)]
    pub async fn headcount(&self) -> Result<i32> {
        use crate::schema::roles;
        use diesel::prelude::*;
        let mut conn = crate::database::connection()?;

        let count: i64 = roles::table
            .filter(roles::team_id.eq(self.id))
            .filter(roles::active.eq(true))
            .filter(roles::person_id.is_not_null())
            .select(diesel::dsl::count_distinct(roles::person_id))
            .first(&mut conn)?;

        Ok(count as i32)
    }

    /// Sum of active effort across this team's roles.
    pub async fn total_effort(&self) -> Result<i32> {
        use crate::schema::{roles, works};
        use crate::models::WorkStatus;
        use diesel::prelude::*;
        let mut conn = connection()?;

        let res = works::table
            .inner_join(roles::table)
            .filter(roles::team_id.eq(self.id))
            .filter(roles::active.eq(true))
            .filter(works::work_status.ne_all(vec![WorkStatus::Cancelled, WorkStatus::Completed]))
            .select(works::effort)
            .load::<i32>(&mut conn)?;

        Ok(res.into_iter().sum())
    }

}

#[derive(Debug, Clone, Deserialize, Serialize, Insertable, InputObject)]
/// Linked from HealthProfile
/// Linked to Trip
#[diesel(table_name = teams)]
pub struct NewTeam {
    pub name_en: String,
    pub name_fr: String,

    pub organization_id: Uuid,
    pub org_tier_id: Uuid,
    pub primary_domain: SkillDomain,
    
    pub description_en: String,
    pub description_fr: String,
}

impl NewTeam {

    pub fn new(
        name_en: String,
        name_fr: String,
        organization_id: Uuid,
        org_tier_id: Uuid,
        primary_domain: SkillDomain,
        description_en: String,
        description_fr: String,
    ) -> Self {
        NewTeam {
            name_en,
            name_fr,
            organization_id,
            org_tier_id,
            primary_domain,
            description_en,
            description_fr,
        }
    }
}
