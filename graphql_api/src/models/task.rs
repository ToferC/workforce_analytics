use std::fmt::Debug;

use chrono::{prelude::*};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use diesel::{self, Insertable, Queryable, ExpressionMethods, BoolExpressionMethods, PgTextExpressionMethods};
use diesel::{RunQueryDsl, QueryDsl};
use uuid::Uuid;
use async_graphql::*;
use async_graphql::dataloader::DataLoader;

use crate::schema::*;
use crate::database::connection;
use crate::graphql::loaders::ProductLoader;

use crate::models::{Priority, SkillDomain, WorkStatus};

use super::{Work, Role, Product, Contract, FinancialSummary, contracts_summary};

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Insertable, AsChangeset, SimpleObject)]
#[graphql(complex)]
#[diesel(table_name = tasks)]
pub struct Task {
    pub id: Uuid,
    pub created_by_role_id: Uuid, // Person
    pub title: String,
    pub domain: SkillDomain,
    pub intended_outcome: String,
    pub final_outcome: Option<String>,
    pub approval_tier: i32,
    pub url: String,
    pub start_datestamp: NaiveDateTime,
    pub target_completion_date: NaiveDateTime,
    pub task_status: WorkStatus,
    pub priority: Priority,
    pub completed_date: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    #[graphql(skip)]
    pub product_id: Option<Uuid>, // Product
    /// Approval workflow state (Proposal 7b). `approval_tier` above stays the
    /// level required; this is the actual state of the approval.
    pub approval_status: ApprovalStatus,
    #[graphql(skip)]
    pub approved_by_user_id: Option<Uuid>,
    /// When the task was approved or rejected.
    pub approved_at: Option<NaiveDateTime>,
    /// Reason captured when a task is REJECTED.
    pub rejection_reason: Option<String>,
}

#[ComplexObject]
impl Task {
    pub async fn work(&self) -> Result<Vec<Work>> {
        Work::get_by_task_id(&self.id)
    }

    /// Work under this task not yet assigned to a role
    pub async fn vacant_work(&self) -> Result<Vec<Work>> {
        Work::get_vacant_by_task_id(&self.id)
    }

    pub async fn effort(&self) -> Result<i32> {
        Work::sum_task_effort(&self.id)
    }

    pub async fn created_by(&self) -> Result<Role> {
        Role::get_by_id(&self.created_by_role_id)
    }

    /// The product this task contributes to, if any
    pub async fn product(&self, ctx: &Context<'_>) -> Result<Option<Product>> {
        match self.product_id {
            Some(id) => Ok(ctx.data_unchecked::<DataLoader<ProductLoader>>().load_one(id).await?),
            None => Ok(None),
        }
    }

    /// Name of the user who approved or rejected the task, if any (Proposal 7b).
    pub async fn approved_by_name(&self) -> Option<String> {
        self.approved_by_user_id
            .and_then(|id| crate::models::User::get_by_id(&id).ok().map(|u| u.name))
    }

    /// True when this task's priority is lower than its parent product's
    /// priority — a planning inconsistency (Proposal 7c). False when the task
    /// has no product.
    pub async fn priority_below_parent(&self) -> Result<bool> {
        match self.product_id {
            Some(pid) => {
                let product = Product::get_by_id(&pid)?;
                Ok(self.priority < product.priority)
            }
            None => Ok(false),
        }
    }

    /// Number of work items under this task whose priority is lower than the
    /// task's own priority (Proposal 7c).
    /// Contracts recorded under this task, earliest start first.
    pub async fn contracts(&self) -> Result<Vec<Contract>> {
        let id = self.id;
        crate::graphql::loaders::off_executor(move || Contract::get_by_task_id(&id)).await
    }

    /// Fiscal-year procurement picture for this task: the current FY's share
    /// of every contract under it (committed spend, so budgeted = projected).
    pub async fn finances(&self) -> Result<FinancialSummary> {
        let id = self.id;
        crate::graphql::loaders::off_executor(move || {
            let contracts = Contract::get_by_task_id(&id)?;
            Ok(contracts_summary(&contracts, chrono::Utc::now().date_naive()))
        })
        .await
    }

    pub async fn work_priority_mismatch_count(&self) -> Result<i32> {
        let work = Work::get_by_task_id(&self.id)?;
        Ok(work.iter().filter(|w| w.priority < self.priority).count() as i32)
    }
}

// Non Graphql
impl Task {
    pub fn create(task: &NewTask) -> Result<Task> {
        let mut conn = connection()?;

        let res = diesel::insert_into(tasks::table)
        .values(task)
        .get_result(&mut conn)?;
        
        Ok(res)
    }
    
    pub fn get_or_create(task: &NewTask) -> Result<Task> {
        let mut conn = connection()?;

        let res = tasks::table
            .filter(tasks::created_by_role_id.eq(&task.created_by_role_id)
                .and(tasks::title.eq(&task.title))
                .and(tasks::target_completion_date.eq(&task.target_completion_date))
            )
            .distinct()
            .first(&mut conn);
        
        let task = match res {
            Ok(p) => p,
            Err(e) => {
                // Task not found
                println!("{:?}", e);
                let p = Task::create(task).expect("Unable to create task");
                p
            }
        };
        Ok(task)
    }

    pub fn get_all() -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let res = tasks::table.load::<Task>(&mut conn)?;
        Ok(res)
    }

    pub fn get_count(count: i64) -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let res = tasks::table
            .limit(count)
            .load::<Task>(&mut conn)?;
        
        Ok(res)
    }

    pub fn get_by_id(id: &Uuid) -> Result<Self> {
        let mut conn = connection()?;
        let res = tasks::table.filter(tasks::id.eq(id)).first(&mut conn)?;
        Ok(res)
    }

    /// Batched lookup for the DataLoader: fetch many tasks in one query.
    pub fn get_by_ids(ids: &[Uuid]) -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let res = tasks::table.filter(tasks::id.eq_any(ids)).load::<Task>(&mut conn)?;
        Ok(res)
    }

    pub fn get_by_assigning_person_id(id: Uuid) -> Result<Vec<Task>> {
        let mut conn = connection()?;

        let res = tasks::table
            .filter(tasks::created_by_role_id.eq(id))
            .load::<Task>(&mut conn)?;

        Ok(res)
    }

    pub fn get_by_product_id(product_id: &Uuid) -> Result<Vec<Task>> {
        let mut conn = connection()?;

        let res = tasks::table
            .filter(tasks::product_id.eq(product_id))
            .order_by(tasks::created_at)
            .load::<Task>(&mut conn)?;

        Ok(res)
    }

    pub fn get_by_title(title: &String) -> Result<Vec<Task>> {
        let mut conn = connection()?;

        let res = tasks::table
            .filter(tasks::title.ilike(format!("%{}%", title)))
            .load::<Task>(&mut conn)?;

        Ok(res)
    }
    
    pub fn update(&self) -> Result<Self> {
        let mut conn = connection()?;

        let res = diesel::update(tasks::table)
        .filter(tasks::id.eq(&self.id))
        .set(self)
        .get_result(&mut conn)?;

        Ok(res)
    }

    // ── Approval workflow (Proposal 7b) ──────────────────────────────────
    // Targeted column updates so a transition sets exactly the approval
    // columns (and can NULL them), without touching the rest of the row.

    /// DRAFT | REJECTED -> PENDING_APPROVAL. Clears any prior rejection reason
    /// and approver so the record reflects the fresh submission.
    pub fn submit_for_approval(id: &Uuid) -> Result<Self> {
        let mut conn = connection()?;
        let res = diesel::update(tasks::table.filter(tasks::id.eq(id)))
            .set((
                tasks::approval_status.eq(ApprovalStatus::PendingApproval),
                tasks::approved_by_user_id.eq::<Option<Uuid>>(None),
                tasks::approved_at.eq::<Option<NaiveDateTime>>(None),
                tasks::rejection_reason.eq::<Option<String>>(None),
            ))
            .get_result(&mut conn)?;
        Ok(res)
    }

    /// PENDING_APPROVAL -> APPROVED, recording approver + time.
    pub fn approve(id: &Uuid, approver_user_id: Option<Uuid>) -> Result<Self> {
        let mut conn = connection()?;
        let res = diesel::update(tasks::table.filter(tasks::id.eq(id)))
            .set((
                tasks::approval_status.eq(ApprovalStatus::Approved),
                tasks::approved_by_user_id.eq(approver_user_id),
                tasks::approved_at.eq(Some(Utc::now().naive_utc())),
                tasks::rejection_reason.eq::<Option<String>>(None),
            ))
            .get_result(&mut conn)?;
        Ok(res)
    }

    /// PENDING_APPROVAL -> REJECTED, recording who, when, and why.
    pub fn reject(id: &Uuid, approver_user_id: Option<Uuid>, reason: String) -> Result<Self> {
        let mut conn = connection()?;
        let res = diesel::update(tasks::table.filter(tasks::id.eq(id)))
            .set((
                tasks::approval_status.eq(ApprovalStatus::Rejected),
                tasks::approved_by_user_id.eq(approver_user_id),
                tasks::approved_at.eq(Some(Utc::now().naive_utc())),
                tasks::rejection_reason.eq(Some(reason)),
            ))
            .get_result(&mut conn)?;
        Ok(res)
    }

    /// Tasks awaiting approval, newest first (capped). Caller scopes these to
    /// what the principal manages.
    pub fn pending_approvals(limit: i64) -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let res = tasks::table
            .filter(tasks::approval_status.eq(ApprovalStatus::PendingApproval))
            .order_by(tasks::updated_at.desc())
            .limit(limit)
            .load::<Task>(&mut conn)?;
        Ok(res)
    }
}

/// Approval workflow state for a Task (Proposal 7b).
#[derive(Debug, Clone, Copy, PartialEq, Eq, DbEnum, Serialize, Deserialize, Enum)]
#[ExistingTypePath = "crate::schema::sql_types::ApprovalStatus"]
pub enum ApprovalStatus {
    Draft,
    PendingApproval,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, Deserialize, Serialize, Insertable, SimpleObject, InputObject)]
#[diesel(table_name = tasks)]
pub struct NewTask {
    pub created_by_role_id: Uuid, // Person
    pub title: String,
    pub domain: SkillDomain,
    pub intended_outcome: String,
    pub approval_tier: i32,
    pub url: String,
    pub start_datestamp: NaiveDateTime,
    pub target_completion_date: NaiveDateTime,
    pub task_status: WorkStatus,
    pub priority: Priority,
    pub product_id: Option<Uuid>, // Product
}

impl NewTask {

    pub fn new(
        created_by_role_id: Uuid, // Person
        title: String,
        domain: SkillDomain,
        intended_outcome: String,
        approval_tier: i32,
        url: String,
        start_datestamp: NaiveDateTime,
        target_completion_date: NaiveDateTime,
        task_status: WorkStatus,
        priority: Priority,
        product_id: Option<Uuid>, // Product
    ) -> Self {
        NewTask {
            created_by_role_id,
            title,
            domain,
            intended_outcome,
            approval_tier,
            url,
            start_datestamp,
            target_completion_date,
            task_status,
            priority,
            product_id,
        }
    }
}
