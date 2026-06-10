use std::fmt::Debug;

use chrono::{prelude::*};
use serde::{Deserialize, Serialize};
use diesel::{self, Insertable, Queryable, ExpressionMethods, BoolExpressionMethods, PgTextExpressionMethods};
use diesel::{RunQueryDsl, QueryDsl};
use uuid::Uuid;
use async_graphql::*;

use crate::schema::*;
use crate::database::connection;

use crate::models::{Organization, Role, SkillDomain, Work, WorkStatus};

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Identifiable, Insertable, AsChangeset, SimpleObject, Associations)]
#[diesel(belongs_to(Organization))]
#[diesel(table_name = products)]
#[graphql(complex)]
/// A product or service delivered by an organization.
/// Work elements are planned under a product with their capability
/// requirements so that people with the required capabilities can be
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

    /// All work elements planned under this product
    pub async fn work(&self) -> Result<Vec<Work>> {
        Work::get_by_product_id(&self.id)
    }

    /// Work elements under this product not yet assigned to a role
    pub async fn vacant_work(&self) -> Result<Vec<Work>> {
        Work::get_vacant_by_product_id(&self.id)
    }

    /// Total effort of active work planned under this product
    pub async fn effort(&self) -> Result<i32> {
        Work::sum_product_effort(&self.id)
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
}

#[derive(Debug, Clone, Deserialize, Serialize, Insertable, InputObject)]
#[table_name = "products"]
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
        }
    }
}
