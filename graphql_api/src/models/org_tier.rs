use std::fmt::Debug;

use chrono::{prelude::*};
use serde::{Deserialize, Serialize};
use diesel::{self, Insertable, Queryable, ExpressionMethods, BoolExpressionMethods, PgTextExpressionMethods};
use diesel::{RunQueryDsl, QueryDsl};
use uuid::Uuid;
use async_graphql::*;

use crate::database::connection;
use crate::schema::*;

use super::{Organization, Person, OrgOwnership, SkillDomain, Team};

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Insertable, AsChangeset, SimpleObject)]
#[graphql(complex)]
#[diesel(table_name = org_tiers)]
#[diesel(belongs_to(Organization))]
/// Represents an organizational level starting at the top (CEO or President's office) as 0
/// and then increasing in tier number as you go deeper into the organization.
/// Used to model an organizational hierarchy independent of people
pub struct OrgTier {
    pub id: Uuid,

    #[graphql(visible = false)]
    pub organization_id: Uuid, // Organization
    pub tier_level: i32,
    pub name_en: String,
    pub name_fr: String,
    pub primary_domain: SkillDomain,

    #[graphql(visible = false)]
    pub parent_tier: Option<Uuid>, // Recursive reference to OrgTier
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub retired_at: Option<NaiveDateTime>,
}

#[ComplexObject]
impl OrgTier {

    pub async fn organization(&self) -> Result<Organization> {
        Organization::get_by_id(&self.organization_id)
    }

    pub async fn parent_organization_tier(&self) -> Result<Option<OrgTier>> {
        match self.parent_tier {
            Some(id) => Ok(Some(OrgTier::get_by_id(&id)?)),
            None => Ok(None),
        }
    }

    pub async fn child_organization_tier(&self) -> Result<Vec<OrgTier>> {
        OrgTier::get_child_org_tiers(&self.id)
    }

    pub async fn owner(&self) -> Result<Person> {
        // Tiers created without an explicit ownership record inherit the
        // nearest ancestor's owner until one is assigned. Never panic here:
        // an unwrap would kill the worker for any tier missing ownership.
        let mut tier_id = self.id;
        let mut parent_tier = self.parent_tier;

        loop {
            match OrgOwnership::get_by_org_tier_id(&tier_id) {
                Ok(org_tier_ownership) => return Person::get_by_id(&org_tier_ownership.owner_id),
                Err(_) => match parent_tier {
                    Some(parent_id) => {
                        let parent = OrgTier::get_by_id(&parent_id)?;
                        tier_id = parent.id;
                        parent_tier = parent.parent_tier;
                    },
                    None => return Err(async_graphql::Error::new("No owner assigned to this org tier")),
                },
            }
        }
    }

    pub async fn teams(&self) -> Result<Vec<Team>> {
        Team::get_by_org_tier_id(&self.id)
    }

    /// Capability heatmap rolled up across all teams under this tier and descendants.
    pub async fn capability_heatmap(&self) -> Result<Vec<crate::models::TeamCapabilityRow>> {
        crate::graphql::query::compute_team_capability_matrix(Some(self.id), None)
    }

    /// Number of distinct people holding active roles under this tier and descendants.
    pub async fn headcount(&self) -> Result<i32> {
        let person_ids = crate::graphql::query::get_person_ids_under_org_tier(&self.id)?;
        Ok(person_ids.len() as i32)
    }

    /// Capability counts rolled up across this tier and its descendants.
    pub async fn capability_counts(&self) -> Result<Vec<crate::models::CapabilityCount>> {
        use crate::schema::{org_tiers, teams, roles, capabilities};
        use diesel::dsl::count;
        use diesel::prelude::*;
        use crate::models::CapabilityLevel;
        let mut conn = connection()?;

        let all_tiers_raw: Vec<(Uuid, Option<Uuid>)> = org_tiers::table
            .select((org_tiers::id, org_tiers::parent_tier))
            .load(&mut conn)?;

        let mut tier_ids: Vec<Uuid> = Vec::new();
        let mut queue = vec![self.id];
        while let Some(current) = queue.pop() {
            tier_ids.push(current);
            for (tid, parent) in &all_tiers_raw {
                if *parent == Some(current) {
                    queue.push(*tid);
                }
            }
        }

        let team_ids: Vec<Uuid> = teams::table
            .filter(teams::org_tier_id.eq_any(&tier_ids))
            .select(teams::id)
            .load::<Uuid>(&mut conn)?;

        let person_ids: Vec<Uuid> = roles::table
            .filter(roles::team_id.eq_any(&team_ids))
            .filter(roles::active.eq(true))
            .filter(roles::person_id.is_not_null())
            .select(roles::person_id)
            .load::<Option<Uuid>>(&mut conn)?
            .into_iter()
            .flatten()
            .collect();

        let res: Vec<(String, SkillDomain, Option<CapabilityLevel>, i64)> = capabilities::table
            .filter(capabilities::person_id.eq_any(&person_ids))
            .filter(capabilities::retired_at.is_null())
            .group_by((capabilities::name_en, capabilities::domain, capabilities::validated_level))
            .select((capabilities::name_en, capabilities::domain, capabilities::validated_level, count(capabilities::id)))
            .order_by((capabilities::name_en, capabilities::validated_level))
            .load::<(String, SkillDomain, Option<CapabilityLevel>, i64)>(&mut conn)?;

        Ok(res.into_iter().map(crate::models::CapabilityCount::from).collect())
    }

    /// Sum of active effort across this tier and descendants.
    pub async fn total_effort(&self) -> Result<i32> {
        use crate::schema::{org_tiers, teams, roles, works};
        use crate::models::WorkStatus;
        use diesel::prelude::*;
        let mut conn = connection()?;

        let all_tiers_raw: Vec<(Uuid, Option<Uuid>)> = org_tiers::table
            .select((org_tiers::id, org_tiers::parent_tier))
            .load(&mut conn)?;

        let mut tier_ids: Vec<Uuid> = Vec::new();
        let mut queue = vec![self.id];
        while let Some(current) = queue.pop() {
            tier_ids.push(current);
            for (tid, parent) in &all_tiers_raw {
                if *parent == Some(current) {
                    queue.push(*tid);
                }
            }
        }

        let team_ids: Vec<Uuid> = teams::table
            .filter(teams::org_tier_id.eq_any(&tier_ids))
            .select(teams::id)
            .load::<Uuid>(&mut conn)?;

        let res = works::table
            .inner_join(roles::table)
            .filter(roles::team_id.eq_any(&team_ids))
            .filter(roles::active.eq(true))
            .filter(works::work_status.ne_all(vec![WorkStatus::Cancelled, WorkStatus::Completed]))
            .select(works::effort)
            .load::<i32>(&mut conn)?;

        Ok(res.into_iter().sum())
    }
}

// Non Graphql
impl OrgTier {
    pub fn create(org_tier: &NewOrgTier) -> Result<OrgTier> {
        let mut conn = connection()?;

        let res = diesel::insert_into(org_tiers::table)
            .values(org_tier)
            .get_result(&mut conn)?;
        
        Ok(res)
    }
    
    pub fn get_or_create(org_tier: &NewOrgTier) -> Result<OrgTier> {
        let mut conn = connection()?;

        let res = org_tiers::table
        .filter(org_tiers::name_en.eq(&org_tier.name_en))
        .distinct()
        .first(&mut conn);
        
        let org_tier = match res {
            Ok(p) => p,
            Err(e) => {
                // OrgTier not found
                if e.to_string() == "NotFound" {
                    println!("{:?}", e);
                }
                let p = OrgTier::create(org_tier).expect("Unable to create org_tier");
                p
            }
        };
        Ok(org_tier)
    }

    pub fn get_all() -> Result<Vec<OrgTier>> {
        let mut conn = connection()?;
        let res = org_tiers::table
            .load::<OrgTier>(&mut conn)?;

        Ok(res)
    }

    pub fn get_count(count: i64) -> Result<Vec<OrgTier>> {
        let mut conn = connection()?;
        let res = org_tiers::table
            .limit(count)
            .order(org_tiers::tier_level)
            .load::<OrgTier>(&mut conn)?;

        Ok(res)
    }

    pub fn get_by_id(id: &Uuid) -> Result<OrgTier> {
        let mut conn = connection()?;

        let res = org_tiers::table
            .filter(org_tiers::id.eq(id))
            .first(&mut conn)?;

        Ok(res)
    }

    pub fn get_by_name(name: &str) -> Result<Vec<OrgTier>> {
        let mut conn = connection()?;

        let res = org_tiers::table
            .filter(org_tiers::name_en.ilike(&name).or(org_tiers::name_fr.ilike(format!("%{}%", name))))
            .load::<OrgTier>(&mut conn)?;

        Ok(res)
    }

    pub fn get_by_org_id(id: &Uuid) -> Result<Vec<OrgTier>> {
        let mut conn = connection()?;

        let res = org_tiers::table
            .filter(org_tiers::organization_id.eq(id))
            .load::<OrgTier>(&mut conn)?;

        Ok(res)
    }

    pub fn get_top_by_org_id(id: &Uuid) -> Result<Vec<OrgTier>> {
        let mut conn = connection()?;

        let res = org_tiers::table
            .filter(org_tiers::organization_id.eq(id))
            .filter(org_tiers::parent_tier.is_null())
            .load::<OrgTier>(&mut conn)?;

        Ok(res)
    }

    pub fn get_child_org_tiers(id: &Uuid) -> Result<Vec<OrgTier>> {
        let mut conn = connection()?;

        let res: Vec<Self> = org_tiers::table
            .filter(org_tiers::parent_tier.eq(id))
            .load::<OrgTier>(&mut conn)?;

        Ok(res)
    }

    pub fn get_by_ids(ids: &Vec<Uuid>) -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let org_tier_ownership = org_tiers::table
            .filter(org_tiers::id.eq_any(ids))
            .load::<OrgTier>(&mut conn)?;

        Ok(org_tier_ownership)
    }
    
    pub fn update(&self) -> Result<OrgTier> {
        let mut conn = connection()?;

        let res = diesel::update(org_tiers::table)
            .filter(org_tiers::id.eq(&self.id))
            .set(self)
            .get_result(&mut conn)?;

        Ok(res)
    }

    /// Clear retired_at to un-retire (AsChangeset skips None, so set NULL).
    pub fn restore(id: &Uuid) -> Result<OrgTier> {
        let mut conn = connection()?;

        let res = diesel::update(org_tiers::table.filter(org_tiers::id.eq(id)))
            .set(org_tiers::retired_at.eq::<Option<NaiveDateTime>>(None))
            .get_result(&mut conn)?;

        Ok(res)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Insertable, InputObject)]
#[diesel(table_name = org_tiers)]
pub struct NewOrgTier {
    pub organization_id: Uuid, // Organization
    pub tier_level: i32,
    pub name_en: String,
    pub name_fr: String,
    pub primary_domain: SkillDomain,
    pub parent_tier: Option<Uuid>, // Recursive reference to OrgTier
}

impl NewOrgTier {

    pub fn new(
        organization_id: Uuid, // Organization
        tier_level: i32,
        name_en: String,
        name_fr: String,
        primary_domain: SkillDomain,
        parent_tier: Option<Uuid>, // Recursive reference to OrgTier
    ) -> Self {
        NewOrgTier {
            organization_id,
            tier_level,
            name_en,
            name_fr,
            primary_domain,
            parent_tier,
        }
    }
}
