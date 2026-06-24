use std::collections::HashMap;

use chrono::NaiveDateTime;
use diesel::dsl::count;
use serde::{Serialize, Deserialize};
use diesel::prelude::*;
use diesel::{self, Insertable, Queryable};
use diesel::{RunQueryDsl, QueryDsl};
//use juniper::{Result};
use uuid::Uuid;

use async_graphql::*;

use crate::database::connection;
use crate::schema::*;

use crate::models::{CapabilityCount, CapabilityLevel, Affiliation, SkillDomain, Publication, Product};

use super::OrgTier;

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Identifiable, AsChangeset, SimpleObject)]
#[table_name = "organizations"]
#[graphql(complex)]
/// Represents an organization as a core structure within which are
/// Person(s), OrgTiers, Publications
pub struct Organization {
    pub id: Uuid,
    pub name_en: String,
    pub name_fr: String,
    pub acronym_en: String,
    pub acronym_fr: String,
    pub org_type: String,
    pub url: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub retired_at: Option<NaiveDateTime>,
}

#[ComplexObject]
impl Organization {

    async fn affiliations(&self) -> Result<Vec<Affiliation>> {
        Affiliation::get_by_home_organization_id(self.id)
    }

    pub async fn publications(&self) -> Result<Vec<Publication>> {
        Publication::get_by_publishing_organization_id(&self.id)
    }

    pub async fn products(&self) -> Result<Vec<Product>> {
        Product::get_by_organization_id(&self.id)
    }

    pub async fn org_tiers(&self) -> Result<Vec<OrgTier>> {
        OrgTier::get_by_org_id(&self.id)
    }

    pub async fn top_org_tier(&self) -> Result<Vec<OrgTier>> {
        OrgTier::get_top_by_org_id(&self.id)
    }
    
    async fn capability_counts(&self) -> Result<Vec<CapabilityCount>> {
        let mut conn = connection().unwrap();

        let res: Vec<(String, SkillDomain, Option<CapabilityLevel>, i64)> = capabilities::table
            .filter(capabilities::organization_id.eq(self.id))
            .group_by((capabilities::domain, capabilities::validated_level, capabilities::name_en))
            .select((capabilities::name_en, capabilities::domain, capabilities::validated_level, count(capabilities::id)))
            .order_by((capabilities::name_en, capabilities::validated_level))
            .load::<(String, SkillDomain, Option<CapabilityLevel>, i64)>(&mut conn)?;

    // Convert res into CapabilityCountStruct
    let mut counts: Vec<CapabilityCount> = Vec::new();

    for r in res {
        let count = CapabilityCount::from(r);
        counts.push(count);
    }

    Ok(counts)
    }
}

impl Organization {
    pub fn create(organization: &NewOrganization) -> Result<Organization> {
        let mut conn = connection()?;

        let res = diesel::insert_into(organizations::table)
            .values(organization)
            .get_result(&mut conn)?;

        Ok(res)
    }

    /// Create an organization together with a starter hierarchy so it is not an
    /// empty shell. A bare organization has no tiers and no owner, which leaves
    /// it invisible in the org chart and with no authority to inherit. This
    /// seeds, in a single transaction:
    ///   * a top-level org tier (level 0, no parent);
    ///   * an executive team under that tier (roles must belong to a team);
    ///   * a vacant "head" role on that team — ownership attaches to the
    ///     position, not a person, so it survives staffing changes;
    ///   * an ownership record making that role the owner of the top tier.
    /// The whole structure commits or rolls back as a unit, so a new
    /// organization is never left half-built.
    pub fn create_with_defaults(organization: &NewOrganization) -> Result<Organization> {
        use crate::models::{NewOrgTier, NewTeam, NewRole, NewOrgOwnership, Team, Role};

        let mut conn = connection()?;

        let org = conn.transaction::<Organization, diesel::result::Error, _>(|conn| {
            let now = chrono::Utc::now().naive_utc();

            let org: Organization = diesel::insert_into(organizations::table)
                .values(organization)
                .get_result(conn)?;

            let tier: OrgTier = diesel::insert_into(org_tiers::table)
                .values(NewOrgTier {
                    organization_id: org.id,
                    tier_level: 0,
                    name_en: "Executive".to_string(),
                    name_fr: "Direction".to_string(),
                    primary_domain: SkillDomain::Governance,
                    parent_tier: None,
                })
                .get_result(conn)?;

            let team: Team = diesel::insert_into(teams::table)
                .values(NewTeam {
                    name_en: "Executive Office".to_string(),
                    name_fr: "Bureau de la direction".to_string(),
                    organization_id: org.id,
                    org_tier_id: tier.id,
                    primary_domain: SkillDomain::Governance,
                    description_en: "Senior leadership of the organization.".to_string(),
                    description_fr: "Direction générale de l'organisation.".to_string(),
                })
                .get_result(conn)?;

            let role: Role = diesel::insert_into(roles::table)
                .values(NewRole {
                    person_id: None,
                    team_id: team.id,
                    title_en: format!("Head, {}", org.name_en),
                    title_fr: format!("Chef, {}", org.name_fr),
                    effort: 1.0,
                    active: true,
                    military_occupation: None,
                    rank: None,
                    occupational_group: None,
                    occupational_level: None,
                    start_datestamp: now,
                    end_date: None,
                })
                .get_result(conn)?;

            diesel::insert_into(org_tier_ownerships::table)
                .values(NewOrgOwnership {
                    owner_role_id: role.id,
                    org_tier_id: tier.id,
                })
                .execute(conn)?;

            Ok(org)
        })?;

        Ok(org)
    }

    pub fn get_by_id(id: &Uuid) -> Result<Organization> {
        let mut conn = connection()?;

        let res = organizations::table.filter(organizations::id.eq(id))
            .first(&mut conn)?;

        Ok(res)
    }

    pub fn get_all() -> Result<Vec<Organization>> {
        let mut conn = connection()?;

        let res = organizations::table
            .load::<Organization>(&mut conn)?;

        Ok(res)
    }

    pub fn get_count(count: i64) -> Result<Vec<Organization>> {
        let mut conn = connection()?;

        let res = organizations::table
            .limit(count)
            .load::<Organization>(&mut conn)?;

        Ok(res)
    }

    pub fn get_by_name(name: String) -> Result<Vec<Organization>> {
        let mut conn = connection()?;

        let res = organizations::table
            .filter(organizations::name_en.ilike(format!("%{}%", name)).or(organizations::name_fr.ilike(format!("%{}%", name))))
            .load::<Organization>(&mut conn)?;

        Ok(res)
    }

    pub fn update(&self) -> Result<Organization> {
        let mut conn = connection()?;

        let res = diesel::update(organizations::table)
            .filter(organizations::id.eq(&self.id))
            .set(self)
            .get_result(&mut conn)?;

        Ok(res)
    }

    /// Clear retired_at to un-retire. update()'s AsChangeset skips None
    /// fields, so clearing a column needs an explicit set to NULL.
    pub fn restore(id: &Uuid) -> Result<Organization> {
        let mut conn = connection()?;

        let res = diesel::update(organizations::table.filter(organizations::id.eq(id)))
            .set(organizations::retired_at.eq::<Option<NaiveDateTime>>(None))
            .get_result(&mut conn)?;

        Ok(res)
    }

    pub fn load_into_hash() -> HashMap<Uuid, Organization> {
        let mut conn = connection().expect("Unable to make connection");

        let res = organizations::table
            .load::<Organization>(&mut conn)
            .expect("Unable to get organizations");

        let mut organizations: HashMap<Uuid, Organization> = HashMap::new();
        for c in res {
            organizations.insert(c.id, c);
        };

        organizations 
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Insertable, InputObject)]
#[table_name = "organizations"]
/// Represents an insertable Organization
pub struct NewOrganization {
    pub name_en: String,
    pub name_fr: String,
    pub acronym_en: String,
    pub acronym_fr: String,
    pub org_type: String,
    pub url: String,
}

impl NewOrganization {
    pub fn new(
        name_en: String,
        name_fr: String,
        acronym_en: String,
        acronym_fr: String,
        org_type: String,
        url: String,

    ) -> Self {
        NewOrganization {
            name_en,
            name_fr,
            acronym_en,
            acronym_fr,
            org_type,
            url,
        }
    }
}