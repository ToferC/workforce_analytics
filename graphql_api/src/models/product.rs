use std::collections::HashMap;
use std::fmt::Debug;

use chrono::{prelude::*};
use serde::{Deserialize, Serialize};
use diesel::{self, Insertable, Queryable, ExpressionMethods, BoolExpressionMethods, PgTextExpressionMethods};
use diesel::{RunQueryDsl, QueryDsl};
use uuid::Uuid;
use async_graphql::*;

use crate::schema::*;
use crate::database::connection;

use crate::models::{CapabilityLevel, Organization, Priority, Role, Skill, SkillDomain, Task, Work, WorkStatus};

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Identifiable, Insertable, AsChangeset, SimpleObject, Associations)]
#[diesel(belongs_to(Organization))]
#[diesel(table_name = products)]
#[graphql(complex)]
/// A product or service delivered by an organization.
/// Tasks flow under a product, and multiple people do Work as part of
/// a task that contributes to the product. Work carries its capability
/// requirement so that people with the required capabilities can be
/// identified and matched to the work.
pub struct Product {
    pub id: Uuid,

    #[graphql(skip)]
    pub organization_id: Uuid, // Organization
    #[graphql(skip)]
    pub product_owner_role_id: Uuid, // Role

    pub name_en: String,
    pub name_fr: String,
    pub description_en: String,
    pub description_fr: String,

    pub primary_domain: SkillDomain,
    pub url: Option<String>,
    pub product_status: WorkStatus,
    pub priority: Priority,

    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub retired_at: Option<NaiveDateTime>,
}

// Graphql
#[ComplexObject]
impl Product {
    pub async fn organization(&self) -> Result<Organization> {
        Organization::get_by_id(&self.organization_id)
    }

    /// The role accountable for delivery of this product
    pub async fn product_owner(&self) -> Result<Role> {
        Role::get_by_id(&self.product_owner_role_id)
    }

    /// The tasks that flow under this product
    pub async fn tasks(&self) -> Result<Vec<Task>> {
        Task::get_by_product_id(&self.id)
    }

    /// All work elements planned under this product's tasks
    pub async fn work(&self) -> Result<Vec<Work>> {
        Work::get_by_product_id(&self.id)
    }

    /// Work elements under this product's tasks not yet assigned to a role
    pub async fn vacant_work(&self) -> Result<Vec<Work>> {
        Work::get_vacant_by_product_id(&self.id)
    }

    /// Total effort of active work planned under this product's tasks
    pub async fn effort(&self) -> Result<i32> {
        Work::sum_product_effort(&self.id)
    }

    /// Aggregated skill demand across all active work under this product.
    /// Groups by skill (or domain where no specific skill is set) and
    /// capability level, returning a count of work items and total effort
    /// at each combination. Excludes cancelled and completed work.
    pub async fn skill_demand(&self) -> Result<Vec<ProductSkillDemand>> {
        Product::get_skill_demand(&self.id)
    }
}

// Non Graphql
impl Product {
    pub fn create(product: &NewProduct) -> Result<Product> {
        let mut conn = connection()?;

        let res = diesel::insert_into(products::table)
            .values(product)
            .get_result(&mut conn)?;

        Ok(res)
    }

    pub fn get_or_create(product: &NewProduct) -> Result<Product> {
        let mut conn = connection()?;

        let res = products::table
            .filter(products::organization_id.eq(&product.organization_id)
                .and(products::name_en.eq(&product.name_en)))
            .distinct()
            .first(&mut conn);

        let product = match res {
            Ok(p) => p,
            Err(e) => {
                // Product not found
                println!("{:?}", e);
                let p = Product::create(product).expect("Unable to create product");
                p
            }
        };
        Ok(product)
    }

    pub fn get_all() -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let res = products::table.load::<Product>(&mut conn)?;
        Ok(res)
    }

    pub fn get_count(count: i64) -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let res = products::table
            .limit(count)
            .load::<Product>(&mut conn)?;
        Ok(res)
    }

    pub fn get_by_id(id: &Uuid) -> Result<Self> {
        let mut conn = connection()?;
        let res = products::table
            .filter(products::id.eq(id))
            .first(&mut conn)?;
        Ok(res)
    }

    pub fn get_by_name(name: &String) -> Result<Vec<Self>> {
        let mut conn = connection()?;

        let res = products::table
            .filter(products::name_en.ilike(format!("%{}%", name)).or(products::name_fr.ilike(format!("%{}%", name))))
            .load::<Product>(&mut conn)?;

        Ok(res)
    }

    pub fn get_by_organization_id(organization_id: &Uuid) -> Result<Vec<Self>> {
        let mut conn = connection()?;

        let res = products::table
            .filter(products::organization_id.eq(organization_id))
            .order_by(products::created_at)
            .load::<Product>(&mut conn)?;

        Ok(res)
    }

    pub fn update(&self) -> Result<Self> {
        let mut conn = connection()?;

        let res = diesel::update(products::table)
            .filter(products::id.eq(&self.id))
            .set(self)
            .get_result(&mut conn)?;

        Ok(res)
    }

    /// Aggregates active work under a product into a skill demand summary.
    /// Two queries: one for work items, one to resolve skill names by id.
    pub fn get_skill_demand(id: &Uuid) -> Result<Vec<ProductSkillDemand>> {
        let all_work = Work::get_by_product_id(id)?;

        // SkillDomain and CapabilityLevel don't implement Hash, so we group
        // with a Vec and linear search. Product work item counts are small
        // enough that O(n²) here is never the bottleneck.
        let mut rows: Vec<(SkillDomain, Uuid, CapabilityLevel, i64, i32)> = Vec::new();

        for work in &all_work {
            if matches!(work.work_status, WorkStatus::Cancelled | WorkStatus::Completed) {
                continue;
            }
            match rows.iter_mut().find(|(d, s, l, _, _)| {
                *d == work.domain && *s == work.skill_id && *l == work.capability_level
            }) {
                Some(entry) => {
                    entry.3 += 1;
                    entry.4 += work.effort;
                }
                None => rows.push((work.domain, work.skill_id, work.capability_level, 1, work.effort)),
            }
        }

        // Resolve skill names in one batch query.
        let skill_ids: Vec<Uuid> = rows.iter().map(|(_, s, _, _, _)| *s).collect();

        let skill_map: HashMap<Uuid, String> = if skill_ids.is_empty() {
            HashMap::new()
        } else {
            Skill::get_by_ids(&skill_ids)?
                .into_iter()
                .map(|s| (s.id, s.name_en))
                .collect()
        };

        let mut demand: Vec<ProductSkillDemand> = rows
            .into_iter()
            .map(|(domain, skill_id, level, work_count, total_effort)| {
                let skill_name = skill_map.get(&skill_id).cloned();
                let name = skill_name.clone().unwrap_or_else(|| format!("{:?}", domain));
                ProductSkillDemand { name, skill_name, domain, level, work_count, total_effort }
            })
            .collect();

        demand.sort_by(|a, b| a.name.cmp(&b.name).then(a.level.cmp(&b.level)));

        Ok(demand)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Insertable, InputObject)]
#[diesel(table_name = products)]
pub struct NewProduct {
    pub organization_id: Uuid, // Organization
    pub product_owner_role_id: Uuid, // Role
    pub name_en: String,
    pub name_fr: String,
    pub description_en: String,
    pub description_fr: String,
    pub primary_domain: SkillDomain,
    pub url: Option<String>,
    pub product_status: WorkStatus,
    pub priority: Priority,
}

/// Aggregated skill demand for a product, derived from its work items.
/// Similar in shape to CapabilityCount and RequirementCount but scoped to
/// the active work planned under a product's tasks.
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct ProductSkillDemand {
    /// Skill name, or the domain label when no specific skill is attached to the work.
    pub name: String,
    /// The specific skill driving this demand, if one is set on the work items.
    pub skill_name: Option<String>,
    pub domain: SkillDomain,
    pub level: CapabilityLevel,
    /// Number of distinct work items requiring this skill at this level.
    pub work_count: i64,
    /// Sum of effort across those work items.
    pub total_effort: i32,
}

impl NewProduct {

    pub fn new(
        organization_id: Uuid, // Organization
        product_owner_role_id: Uuid, // Role
        name_en: String,
        name_fr: String,
        description_en: String,
        description_fr: String,
        primary_domain: SkillDomain,
        url: Option<String>,
        product_status: WorkStatus,
        priority: Priority,
    ) -> Self {
        NewProduct {
            organization_id,
            product_owner_role_id,
            name_en,
            name_fr,
            description_en,
            description_fr,
            primary_domain,
            url,
            product_status,
            priority,
        }
    }
}
