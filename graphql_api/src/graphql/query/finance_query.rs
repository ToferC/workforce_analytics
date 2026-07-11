use async_graphql::*;
use uuid::Uuid;

use crate::models::{Contract, PayRate};
use crate::common_utils::{RoleGuard, UserRole};
use crate::graphql::loaders::off_executor;

#[derive(Default)]
pub struct FinanceQuery;

#[Object]
impl FinanceQuery {
    /// The full pay-rate table, every effective date included. The rate in
    /// force for a classification is the row with the latest effective date
    /// that has passed.
    #[graphql(name = "payRates", guard = "RoleGuard::new(UserRole::User)")]
    pub async fn pay_rates(&self, _context: &Context<'_>) -> Result<Vec<PayRate>> {
        off_executor(PayRate::get_all).await
    }

    /// A single contract by id.
    #[graphql(name = "contractById", guard = "RoleGuard::new(UserRole::User)")]
    pub async fn contract_by_id(&self, _context: &Context<'_>, id: Uuid) -> Result<Contract> {
        off_executor(move || Contract::get_by_id(&id)).await
    }
}
