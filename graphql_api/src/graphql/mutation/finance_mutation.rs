use async_graphql::*;
use uuid::Uuid;

use crate::models::{Contract, ContractUpdate, NewContract, NewPayRate, PayRate};
use crate::common_utils::{RoleGuard, UserRole, is_admin, is_operator};
use crate::graphql::loaders::off_executor;

#[derive(Default)]
pub struct FinanceMutation;

#[Object]
impl FinanceMutation {
    /// Add a pay rate for a classification. Superseding an existing rate is
    /// done by inserting a row with a later effective date, preserving
    /// history; rates are never edited in place.
    #[graphql(
        name = "createPayRate",
        guard = "RoleGuard::new(UserRole::Admin)",
        visible = "is_admin",
    )]
    pub async fn create_pay_rate(
        &self,
        _context: &Context<'_>,
        pay_rate_data: NewPayRate,
    ) -> Result<PayRate> {
        off_executor(move || PayRate::create(&pay_rate_data)).await
    }

    /// Record a contract under a task.
    #[graphql(
        name = "createContract",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn create_contract(
        &self,
        _context: &Context<'_>,
        contract_data: NewContract,
    ) -> Result<Contract> {
        off_executor(move || Contract::create(&contract_data)).await
    }

    /// Update a contract (period, value, status, reference details). Only the
    /// provided fields change.
    #[graphql(
        name = "updateContract",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn update_contract(
        &self,
        _context: &Context<'_>,
        contract_data: ContractUpdate,
    ) -> Result<Contract> {
        off_executor(move || Contract::update(&contract_data)).await
    }

    /// Remove a contract recorded in error.
    #[graphql(
        name = "deleteContract",
        guard = "RoleGuard::new(UserRole::Operator)",
        visible = "is_operator",
    )]
    pub async fn delete_contract(&self, _context: &Context<'_>, id: Uuid) -> Result<bool> {
        off_executor(move || Contract::delete(&id)).await
    }
}
