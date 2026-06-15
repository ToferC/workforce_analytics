use async_graphql::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::models::{Product, NewProduct, SkillDomain, WorkStatus};
use crate::common_utils::{UserRole, is_operator, RoleGuard};

#[derive(Default)]
pub struct ProductMutation;

#[Object]
impl ProductMutation {

    #[graphql(
        name = "createProduct",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn create_product(
        &self,
        _context: &Context<'_>,
        data: NewProduct,
    ) -> Result<Product> {
        let product = Product::create(&data)?;
        Ok(product)
    }

    #[graphql(
        name = "updateProduct",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn update_product(
        &self,
        _context: &Context<'_>,
        data: ProductData,
    ) -> Result<Product> {
        let mut product = Product::get_by_id(&data.id)?;

        if let Some(s) = data.product_owner_role_id {
            product.product_owner_role_id = s;
        };

        if let Some(s) = data.name_en {
            product.name_en = s;
        };

        if let Some(s) = data.name_fr {
            product.name_fr = s;
        };

        if let Some(s) = data.description_en {
            product.description_en = s;
        };

        if let Some(s) = data.description_fr {
            product.description_fr = s;
        };

        if let Some(s) = data.primary_domain {
            product.primary_domain = s;
        };

        if let Some(s) = data.url {
            product.url = Some(s);
        };

        if let Some(s) = data.product_status {
            product.product_status = s;
        };

        product.update()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, InputObject)]
/// InputObject for Product with Option fields - only include the ones you want to update
pub struct ProductData {
    pub id: Uuid,
    pub product_owner_role_id: Option<Uuid>,
    pub name_en: Option<String>,
    pub name_fr: Option<String>,
    pub description_en: Option<String>,
    pub description_fr: Option<String>,
    pub primary_domain: Option<SkillDomain>,
    pub url: Option<String>,
    pub product_status: Option<WorkStatus>,
}
