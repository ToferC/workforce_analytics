use async_graphql::*;

use crate::models::{Product};
use uuid::Uuid;

use crate::common_utils::{RoleGuard, UserRole};

#[derive(Default)]
pub struct ProductQuery;

#[Object]
impl ProductQuery {

    // Products

    #[graphql(name = "products", guard = "RoleGuard::new(UserRole::User)")]
    /// Accepts an argument of "count" and returns a vector of {count} products
    pub async fn get_count_products(&self, _context: &Context<'_>, count: i64) -> Result<Vec<Product>> {

        Product::get_count(count)
    }

    #[graphql(name = "allProducts", guard = "RoleGuard::new(UserRole::User)")]
    /// Returns a vector of all products
    pub async fn all_products(
        &self,
        _context: &Context<'_>,) -> Result<Vec<Product>> {

        Product::get_all()
    }

    #[graphql(name = "productById", guard = "RoleGuard::new(UserRole::User)")]
    pub async fn product_by_id(
        &self,
        _context: &Context<'_>,
        id: Uuid,
    ) -> Result<Product> {

        Product::get_by_id(&id)
    }

    #[graphql(name = "productsByName", guard = "RoleGuard::new(UserRole::User)")]
    /// Returns a vector of products matching some part of a provided name
    pub async fn products_by_name(
        &self,
        _context: &Context<'_>,
        name: String,
    ) -> Result<Vec<Product>> {

        Product::get_by_name(&name)
    }

    #[graphql(name = "productsByOrganizationId", guard = "RoleGuard::new(UserRole::User)")]
    /// Returns a vector of products delivered by a specific organization
    pub async fn products_by_organization_id(
        &self,
        _context: &Context<'_>,
        organization_id: Uuid,
    ) -> Result<Vec<Product>> {

        Product::get_by_organization_id(&organization_id)
    }
}
