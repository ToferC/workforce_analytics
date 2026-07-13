use async_graphql::*;
use uuid::Uuid;

use crate::models::{BudgetAllocation, Contract, PayRate, current_fiscal_year};
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

    /// Budget allocations for a fiscal year (its starting year; defaults to
    /// the current one).
    #[graphql(name = "budgetAllocations", guard = "RoleGuard::new(UserRole::User)")]
    pub async fn budget_allocations(
        &self,
        _context: &Context<'_>,
        fiscal_year: Option<i32>,
    ) -> Result<Vec<BudgetAllocation>> {
        off_executor(move || {
            let fy = fiscal_year
                .unwrap_or_else(|| current_fiscal_year(chrono::Utc::now().date_naive()));
            BudgetAllocation::get_for_fiscal_year(fy)
        })
        .await
    }

    /// A single contract by id.
    #[graphql(name = "contractById", guard = "RoleGuard::new(UserRole::User)")]
    pub async fn contract_by_id(&self, _context: &Context<'_>, id: Uuid) -> Result<Contract> {
        off_executor(move || Contract::get_by_id(&id)).await
    }
}
