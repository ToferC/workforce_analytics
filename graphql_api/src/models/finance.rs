//! Financial module: classification pay rates, contract spend under tasks,
//! and the fiscal-year costing math that ties them to roles and teams.
//!
//! Design: costs are always *computed* from data that already exists — the
//! pay-rate table, a role's classification, the `RoleAssignment` tenure
//! ledger, and contract periods. Nothing is written when a role is filled or
//! vacated, so the numbers can never go stale. Money is integer cents
//! (BIGINT / i64) everywhere; the Government of Canada fiscal year runs
//! April 1 to March 31.

use chrono::{prelude::*, Duration};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use diesel::{self, Insertable, Queryable, AsChangeset, ExpressionMethods, JoinOnDsl, NullableExpressionMethods};
use diesel::{RunQueryDsl, QueryDsl};
use uuid::Uuid;
use async_graphql::*;

use crate::schema::*;
use crate::database::connection;
use crate::models::{OccupationalGroup, Rank};

/// Days used to convert an annual rate to a daily accrual. A fixed civil-day
/// denominator keeps the math obvious and stable across leap years; the error
/// versus true calendar length is under 0.3% and identical for every role.
const DAYS_PER_YEAR: i64 = 365;

// ---------------------------------------------------------------------------
// Fiscal year helpers
// ---------------------------------------------------------------------------

/// April 1 of the fiscal year containing `date`.
pub fn fiscal_year_start(date: NaiveDate) -> NaiveDate {
    let year = if date.month() >= 4 { date.year() } else { date.year() - 1 };
    NaiveDate::from_ymd_opt(year, 4, 1).expect("valid fiscal year start")
}

/// March 31 ending the fiscal year containing `date`.
pub fn fiscal_year_end(date: NaiveDate) -> NaiveDate {
    let start = fiscal_year_start(date);
    NaiveDate::from_ymd_opt(start.year() + 1, 3, 31).expect("valid fiscal year end")
}

/// Label like "2026-27" for the fiscal year containing `date`.
pub fn fiscal_year_label(date: NaiveDate) -> String {
    let start = fiscal_year_start(date);
    format!("{}-{:02}", start.year(), (start.year() + 1) % 100)
}

/// Inclusive day-count of the overlap between two closed date ranges, or 0.
pub fn overlap_days(
    a_start: NaiveDate,
    a_end: NaiveDate,
    b_start: NaiveDate,
    b_end: NaiveDate,
) -> i64 {
    let start = a_start.max(b_start);
    let end = a_end.min(b_end);
    if end < start {
        0
    } else {
        (end - start).num_days() + 1
    }
}

/// Linear proration: the share of `total_cents` earned by the window
/// [win_start, win_end] out of the full period [period_start, period_end].
pub fn prorate_cents(
    total_cents: i64,
    period_start: NaiveDate,
    period_end: NaiveDate,
    win_start: NaiveDate,
    win_end: NaiveDate,
) -> i64 {
    let period_days = (period_end - period_start).num_days() + 1;
    if period_days <= 0 {
        return 0;
    }
    let shared = overlap_days(period_start, period_end, win_start, win_end);
    total_cents * shared / period_days
}

// ---------------------------------------------------------------------------
// FinancialSummary — the one shape every altitude reports
// ---------------------------------------------------------------------------

/// Fiscal-year cost picture for a role, task, or team. All values are cents.
#[derive(Debug, Clone, Default, Serialize, Deserialize, SimpleObject)]
pub struct FinancialSummary {
    /// Fiscal year the numbers cover, e.g. "2026-27".
    pub fiscal_year: String,
    /// Full-year cost if everything runs as planned: salary for every day the
    /// role exists in the FY (occupied or not), plus contract shares.
    pub budgeted_cents: i64,
    /// Expected actual spend to March 31: salary accrued over occupied days so
    /// far plus, if occupied today, the remainder of the year; plus contract
    /// shares (contracts are committed spend, so they project in full).
    pub projected_cents: i64,
    /// Budget not expected to be spent — for roles this is exactly the salary
    /// lapse caused by vacancy.
    pub lapse_cents: i64,
}

impl FinancialSummary {
    pub fn add(&mut self, other: &FinancialSummary) {
        self.budgeted_cents += other.budgeted_cents;
        self.projected_cents += other.projected_cents;
        self.lapse_cents += other.lapse_cents;
    }
}

/// Fiscal-year salary picture for one role priced at `annual_rate_cents`.
///
/// * `role_window` — the role's own lifetime (start, optional end).
/// * `assignments` — occupancy periods from the RoleAssignment ledger.
/// * `today` — evaluation date; accrual is counted through today and the
///   projection runs from tomorrow to fiscal year end while occupied.
pub fn salary_summary(
    annual_rate_cents: i64,
    role_window: (NaiveDate, Option<NaiveDate>),
    assignments: &[(NaiveDate, Option<NaiveDate>)],
    today: NaiveDate,
) -> FinancialSummary {
    let fy_start = fiscal_year_start(today);
    let fy_end = fiscal_year_end(today);

    // Clip the role's lifetime to the fiscal year: the budget only covers
    // days the position exists.
    let budget_start = role_window.0.max(fy_start);
    let budget_end = role_window.1.unwrap_or(fy_end).min(fy_end);

    let daily = |days: i64| annual_rate_cents * days / DAYS_PER_YEAR;

    let budget_days = overlap_days(budget_start, budget_end, fy_start, fy_end);
    let budgeted_cents = daily(budget_days);

    // Accrued: occupied days in the FY up to and including today.
    let mut occupied_days = 0i64;
    let mut occupied_today = false;
    for (a_start, a_end) in assignments {
        let closed_end = a_end.unwrap_or(today).min(today);
        occupied_days += overlap_days(*a_start, closed_end, budget_start, budget_end.min(today));
        if *a_start <= today && a_end.map_or(true, |e| e >= today) {
            occupied_today = true;
        }
    }

    // Projection: if someone is in the seat today, assume they stay to
    // March 31 (or the role's own end date if sooner).
    let mut projected_days = occupied_days;
    if occupied_today && today < budget_end {
        projected_days += overlap_days(
            today + Duration::days(1),
            budget_end,
            fy_start,
            fy_end,
        );
    }

    let projected_cents = daily(projected_days);

    FinancialSummary {
        fiscal_year: fiscal_year_label(today),
        budgeted_cents,
        projected_cents,
        lapse_cents: (budgeted_cents - projected_cents).max(0),
    }
}

// ---------------------------------------------------------------------------
// PayRate
// ---------------------------------------------------------------------------

/// Annual salary for a classification: civilian (occupational group + level)
/// or military (rank) — exactly one, mirroring the duality on Role. Rates are
/// superseded by inserting a row with a later effective date, never edited in
/// place, so history is preserved.
#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Insertable, AsChangeset, SimpleObject)]
#[diesel(table_name = pay_rates)]
pub struct PayRate {
    pub id: Uuid,
    pub occupational_group: Option<OccupationalGroup>,
    pub occupational_level: Option<i32>,
    pub rank: Option<Rank>,
    pub annual_rate_cents: i64,
    pub effective_date: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Deserialize, Serialize, Insertable, InputObject)]
#[diesel(table_name = pay_rates)]
pub struct NewPayRate {
    pub occupational_group: Option<OccupationalGroup>,
    pub occupational_level: Option<i32>,
    pub rank: Option<Rank>,
    pub annual_rate_cents: i64,
    pub effective_date: NaiveDateTime,
}

impl PayRate {
    pub fn create(rate: &NewPayRate) -> Result<PayRate> {
        let civilian = rate.occupational_group.is_some() && rate.occupational_level.is_some();
        let military = rate.rank.is_some();
        if civilian == military {
            return Err(Error::new(
                "A pay rate prices exactly one classification: either an occupational group and level, or a rank",
            ));
        }
        if rate.annual_rate_cents < 0 {
            return Err(Error::new("Annual rate cannot be negative"));
        }
        let mut conn = connection()?;
        let res = diesel::insert_into(pay_rates::table)
            .values(rate)
            .get_result(&mut conn)?;
        Ok(res)
    }

    pub fn get_all() -> Result<Vec<PayRate>> {
        let mut conn = connection()?;
        let res = pay_rates::table
            .order((
                pay_rates::rank.asc(),
                pay_rates::occupational_group.asc(),
                pay_rates::occupational_level.asc(),
                pay_rates::effective_date.desc(),
            ))
            .load::<PayRate>(&mut conn)?;
        Ok(res)
    }

    /// All rates in force at `as_of` (their effective_date has passed),
    /// newest first, for in-memory pricing of many roles at once.
    pub fn get_effective(as_of: NaiveDateTime) -> Result<Vec<PayRate>> {
        let mut conn = connection()?;
        let res = pay_rates::table
            .filter(pay_rates::effective_date.le(as_of))
            .order(pay_rates::effective_date.desc())
            .load::<PayRate>(&mut conn)?;
        Ok(res)
    }

    /// Price a classification from a pre-loaded `get_effective` list (which is
    /// newest-first, so the first match is the rate in force).
    pub fn rate_from<'a>(
        rates: &'a [PayRate],
        occupational_group: Option<OccupationalGroup>,
        occupational_level: Option<i32>,
        rank: Option<Rank>,
    ) -> Option<i64> {
        rates
            .iter()
            .find(|r| {
                if rank.is_some() {
                    r.rank == rank
                } else {
                    r.rank.is_none()
                        && r.occupational_group == occupational_group
                        && r.occupational_level == occupational_level
                }
            })
            .map(|r| r.annual_rate_cents)
    }
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, DbEnum, Serialize, Deserialize, Enum)]
#[ExistingTypePath = "crate::schema::sql_types::ContractStatus"]
pub enum ContractStatus {
    Planned,
    Active,
    Closed,
}

/// Procurement spend recorded under a Task. The value is recognized linearly
/// across [start_date, end_date]; a fiscal year's share is its day-overlap
/// with that period. Amendments are additional contracts (same reference
/// number with a suffix, by convention).
#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Insertable, AsChangeset, SimpleObject)]
#[diesel(table_name = contracts)]
pub struct Contract {
    pub id: Uuid,
    pub task_id: Uuid,
    pub reference_number: String,
    pub vendor: String,
    pub description: String,
    pub start_date: NaiveDateTime,
    pub end_date: NaiveDateTime,
    pub total_value_cents: i64,
    pub status: ContractStatus,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Deserialize, Serialize, Insertable, InputObject)]
#[diesel(table_name = contracts)]
pub struct NewContract {
    pub task_id: Uuid,
    pub reference_number: String,
    pub vendor: String,
    pub description: String,
    pub start_date: NaiveDateTime,
    pub end_date: NaiveDateTime,
    pub total_value_cents: i64,
    pub status: ContractStatus,
}

#[derive(Debug, Clone, Deserialize, Serialize, AsChangeset, InputObject)]
#[diesel(table_name = contracts)]
pub struct ContractUpdate {
    // `id` is the table's primary key, which AsChangeset skips automatically.
    pub id: Uuid,
    pub reference_number: Option<String>,
    pub vendor: Option<String>,
    pub description: Option<String>,
    pub start_date: Option<NaiveDateTime>,
    pub end_date: Option<NaiveDateTime>,
    pub total_value_cents: Option<i64>,
    pub status: Option<ContractStatus>,
}

impl Contract {
    pub fn create(contract: &NewContract) -> Result<Contract> {
        if contract.end_date < contract.start_date {
            return Err(Error::new("Contract end date cannot precede its start date"));
        }
        if contract.total_value_cents < 0 {
            return Err(Error::new("Contract value cannot be negative"));
        }
        if contract.reference_number.trim().is_empty() {
            return Err(Error::new("Contract reference number is required"));
        }
        let mut conn = connection()?;
        let res = diesel::insert_into(contracts::table)
            .values(contract)
            .get_result(&mut conn)?;
        Ok(res)
    }

    pub fn update(update: &ContractUpdate) -> Result<Contract> {
        let mut conn = connection()?;
        let res: Contract = diesel::update(contracts::table.filter(contracts::id.eq(update.id)))
            .set(update)
            .get_result(&mut conn)?;
        if res.end_date < res.start_date {
            return Err(Error::new("Contract end date cannot precede its start date"));
        }
        Ok(res)
    }

    pub fn delete(id: &Uuid) -> Result<bool> {
        let mut conn = connection()?;
        let n = diesel::delete(contracts::table.filter(contracts::id.eq(id))).execute(&mut conn)?;
        Ok(n > 0)
    }

    pub fn get_by_id(id: &Uuid) -> Result<Contract> {
        let mut conn = connection()?;
        let res = contracts::table.filter(contracts::id.eq(id)).first(&mut conn)?;
        Ok(res)
    }

    pub fn get_by_task_id(task_id: &Uuid) -> Result<Vec<Contract>> {
        let mut conn = connection()?;
        let res = contracts::table
            .filter(contracts::task_id.eq(task_id))
            .order(contracts::start_date.asc())
            .load::<Contract>(&mut conn)?;
        Ok(res)
    }

    /// Contracts under every task created by one of the team's roles — the
    /// team-level procurement picture.
    pub fn get_by_team_id(team_id: &Uuid) -> Result<Vec<Contract>> {
        let mut conn = connection()?;
        let res = contracts::table
            .inner_join(tasks::table.on(tasks::id.eq(contracts::task_id)))
            .inner_join(roles::table.on(roles::id.eq(tasks::created_by_role_id)))
            .filter(roles::team_id.eq(team_id))
            .select(contracts::all_columns)
            .load::<Contract>(&mut conn)?;
        Ok(res)
    }

    /// This contract's share of the fiscal year containing `today`.
    pub fn fiscal_year_share_cents(&self, today: NaiveDate) -> i64 {
        prorate_cents(
            self.total_value_cents,
            self.start_date.date(),
            self.end_date.date(),
            fiscal_year_start(today),
            fiscal_year_end(today),
        )
    }
}

/// Starting year of the fiscal year containing `date` (2026 = FY 2026-27).
pub fn current_fiscal_year(date: NaiveDate) -> i32 {
    fiscal_year_start(date).year()
}

// ---------------------------------------------------------------------------
// BudgetAllocation
// ---------------------------------------------------------------------------

/// A dollar envelope granted to an org tier for one fiscal year. Set at L1
/// and distributed ("rolled down") to L2/L3 children as their own rows. One
/// row per tier per fiscal year; setting again replaces the amount.
#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Insertable, AsChangeset, SimpleObject)]
#[diesel(table_name = budget_allocations)]
pub struct BudgetAllocation {
    pub id: Uuid,
    pub org_tier_id: Uuid,
    /// Starting year of the fiscal year (2026 = FY 2026-27).
    pub fiscal_year: i32,
    pub amount_cents: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl BudgetAllocation {
    /// Insert or replace the allocation for (tier, fiscal year).
    pub fn set(org_tier_id: &Uuid, fiscal_year: i32, amount_cents: i64) -> Result<BudgetAllocation> {
        if amount_cents < 0 {
            return Err(Error::new("Allocation cannot be negative"));
        }
        let mut conn = connection()?;
        let now = chrono::Utc::now().naive_utc();
        let res = diesel::insert_into(budget_allocations::table)
            .values((
                budget_allocations::org_tier_id.eq(org_tier_id),
                budget_allocations::fiscal_year.eq(fiscal_year),
                budget_allocations::amount_cents.eq(amount_cents),
            ))
            .on_conflict((budget_allocations::org_tier_id, budget_allocations::fiscal_year))
            .do_update()
            .set((
                budget_allocations::amount_cents.eq(amount_cents),
                budget_allocations::updated_at.eq(now),
            ))
            .get_result(&mut conn)?;
        Ok(res)
    }

    pub fn get_for_fiscal_year(fiscal_year: i32) -> Result<Vec<BudgetAllocation>> {
        let mut conn = connection()?;
        let res = budget_allocations::table
            .filter(budget_allocations::fiscal_year.eq(fiscal_year))
            .load::<BudgetAllocation>(&mut conn)?;
        Ok(res)
    }
}

/// Fiscal-year summary over a set of contracts. Contract value is committed
/// spend, so budgeted and projected are the same number and nothing lapses.
pub fn contracts_summary(contracts: &[Contract], today: NaiveDate) -> FinancialSummary {
    let share: i64 = contracts.iter().map(|c| c.fiscal_year_share_cents(today)).sum();
    FinancialSummary {
        fiscal_year: fiscal_year_label(today),
        budgeted_cents: share,
        projected_cents: share,
        lapse_cents: 0,
    }
}

/// One org tier's fiscal-year financial picture, for the analytics rollup.
/// Salary and contract amounts cover the tier's whole subtree (teams attached
/// to it or any descendant); allocation fields expose the budget envelope and
/// how much of it has been rolled down to direct children.
#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct OrgTierFinancialRow {
    pub org_tier_id: Uuid,
    pub name_en: String,
    pub name_fr: String,
    pub tier_level: i32,
    pub parent_id: Option<Uuid>,
    /// Fiscal year the numbers cover, e.g. "2026-27".
    pub fiscal_year: String,
    /// This tier's own budget allocation for the fiscal year, if one is set.
    pub allocation_cents: Option<i64>,
    /// Sum of the allocations set on this tier's direct children — how much
    /// of the envelope has been rolled down.
    pub child_allocated_cents: i64,
    /// Subtree salary budget plus contract share.
    pub budgeted_cents: i64,
    /// Subtree projected spend to March 31 (salary accrual + projection,
    /// plus committed contract share).
    pub projected_cents: i64,
    /// Subtree vacancy lapse.
    pub lapse_cents: i64,
    /// Contract portion of the subtree numbers.
    pub contract_cents: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn fiscal_year_boundaries() {
        assert_eq!(fiscal_year_start(d(2026, 7, 11)), d(2026, 4, 1));
        assert_eq!(fiscal_year_end(d(2026, 7, 11)), d(2027, 3, 31));
        // Jan-Mar belong to the previous fiscal year.
        assert_eq!(fiscal_year_start(d(2026, 2, 1)), d(2025, 4, 1));
        assert_eq!(fiscal_year_end(d(2026, 2, 1)), d(2026, 3, 31));
        assert_eq!(fiscal_year_label(d(2026, 7, 11)), "2026-27");
        assert_eq!(fiscal_year_label(d(2026, 2, 1)), "2025-26");
        // Boundary days.
        assert_eq!(fiscal_year_start(d(2026, 4, 1)), d(2026, 4, 1));
        assert_eq!(fiscal_year_start(d(2026, 3, 31)), d(2025, 4, 1));
    }

    #[test]
    fn current_fiscal_year_is_start_year() {
        assert_eq!(current_fiscal_year(d(2026, 7, 11)), 2026);
        assert_eq!(current_fiscal_year(d(2026, 3, 31)), 2025);
        assert_eq!(current_fiscal_year(d(2026, 4, 1)), 2026);
    }

    #[test]
    fn overlap_day_math() {
        // Disjoint.
        assert_eq!(overlap_days(d(2026, 1, 1), d(2026, 1, 31), d(2026, 2, 1), d(2026, 2, 28)), 0);
        // Identical single day is inclusive.
        assert_eq!(overlap_days(d(2026, 1, 1), d(2026, 1, 1), d(2026, 1, 1), d(2026, 1, 1)), 1);
        // Partial overlap.
        assert_eq!(overlap_days(d(2026, 1, 1), d(2026, 1, 10), d(2026, 1, 6), d(2026, 1, 20)), 5);
    }

    #[test]
    fn proration_is_linear_by_day_overlap() {
        // A $365,000.00 contract over exactly one non-leap year: $1,000/day.
        let total = 36_500_000i64;
        // Whole period inside window.
        assert_eq!(
            prorate_cents(total, d(2026, 4, 1), d(2027, 3, 31), d(2026, 4, 1), d(2027, 3, 31)),
            total
        );
        // 30 days of overlap.
        assert_eq!(
            prorate_cents(total, d(2026, 4, 1), d(2027, 3, 31), d(2026, 4, 1), d(2026, 4, 30)),
            30 * 100_000
        );
        // No overlap.
        assert_eq!(
            prorate_cents(total, d(2024, 4, 1), d(2025, 3, 31), d(2026, 4, 1), d(2027, 3, 31)),
            0
        );
    }

    #[test]
    fn salary_occupied_all_year_projects_full_budget() {
        // $100,000.00/yr role, existed before the FY, no end; occupied since
        // long before the FY with an open assignment.
        let s = salary_summary(
            10_000_000,
            (d(2020, 1, 1), None),
            &[(d(2021, 6, 1), None)],
            d(2026, 7, 11),
        );
        assert_eq!(s.fiscal_year, "2026-27");
        // Full FY budget: 365 days at the daily rate.
        assert_eq!(s.budgeted_cents, 10_000_000 * 365 / 365);
        assert_eq!(s.projected_cents, s.budgeted_cents);
        assert_eq!(s.lapse_cents, 0);
    }

    #[test]
    fn salary_vacant_role_projects_only_accrued() {
        // Occupied for the first 30 days of the FY (Apr 1-30), vacant since.
        let today = d(2026, 7, 11);
        let s = salary_summary(
            10_000_000,
            (d(2020, 1, 1), None),
            &[(d(2025, 1, 1), Some(d(2026, 4, 30)))],
            today,
        );
        let daily = 10_000_000 / 365;
        assert_eq!(s.projected_cents, 10_000_000 * 30 / 365);
        // Lapse is the rest of the year.
        assert_eq!(s.lapse_cents, s.budgeted_cents - s.projected_cents);
        assert!(s.lapse_cents > daily * 300);
    }

    #[test]
    fn salary_never_occupied_is_pure_lapse() {
        let s = salary_summary(
            10_000_000,
            (d(2020, 1, 1), None),
            &[],
            d(2026, 7, 11),
        );
        assert_eq!(s.projected_cents, 0);
        assert_eq!(s.lapse_cents, s.budgeted_cents);
    }

    #[test]
    fn salary_mid_year_hire_projects_from_start_to_fy_end() {
        // Hired July 1, open-ended: projection = Jul 1 .. Mar 31.
        let today = d(2026, 7, 11);
        let s = salary_summary(
            10_000_000,
            (d(2020, 1, 1), None),
            &[(d(2026, 7, 1), None)],
            today,
        );
        let expected_days = overlap_days(d(2026, 7, 1), d(2027, 3, 31), d(2026, 4, 1), d(2027, 3, 31));
        assert_eq!(s.projected_cents, 10_000_000 * expected_days / 365);
        assert!(s.lapse_cents > 0); // April-June vacancy lapsed.
    }

    #[test]
    fn salary_role_created_mid_year_budgets_partial_year() {
        // Role itself starts Oct 1: budget covers Oct 1 - Mar 31 only.
        let s = salary_summary(
            10_000_000,
            (d(2026, 10, 1), None),
            &[],
            d(2026, 7, 11),
        );
        let expected_days = overlap_days(d(2026, 10, 1), d(2027, 3, 31), d(2026, 4, 1), d(2027, 3, 31));
        assert_eq!(s.budgeted_cents, 10_000_000 * expected_days / 365);
    }

    #[test]
    fn contract_fy_share_spans_fiscal_years() {
        // Jan 1 2026 - Dec 31 2026 straddles FY 2025-26 and 2026-27.
        let c = Contract {
            id: Uuid::new_v4(),
            task_id: Uuid::new_v4(),
            reference_number: "C-1".into(),
            vendor: "V".into(),
            description: String::new(),
            start_date: d(2026, 1, 1).and_hms_opt(0, 0, 0).unwrap(),
            end_date: d(2026, 12, 31).and_hms_opt(0, 0, 0).unwrap(),
            total_value_cents: 36_500_000, // $1,000/day over 365 days
            status: ContractStatus::Active,
            created_at: d(2026, 1, 1).and_hms_opt(0, 0, 0).unwrap(),
            updated_at: d(2026, 1, 1).and_hms_opt(0, 0, 0).unwrap(),
        };
        // Within FY 2026-27: Apr 1 - Dec 31 = 275 days.
        assert_eq!(c.fiscal_year_share_cents(d(2026, 7, 11)), 275 * 100_000);
        // Within FY 2025-26: Jan 1 - Mar 31 = 90 days.
        assert_eq!(c.fiscal_year_share_cents(d(2026, 2, 1)), 90 * 100_000);
    }
}
