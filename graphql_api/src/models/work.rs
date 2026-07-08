use std::fmt::Debug;

use chrono::{prelude::*};
use diesel_derive_enum::DbEnum;
use rand::Rng;
use rand::distributions::Standard;
use rand::prelude::Distribution;
use serde::{Deserialize, Serialize};
use diesel::{self, Insertable, Queryable, ExpressionMethods, BoolExpressionMethods};
use diesel::{RunQueryDsl, QueryDsl};
use uuid::Uuid;
use async_graphql::*;
use async_graphql::dataloader::DataLoader;

use crate::schema::*;
use crate::models::{SkillDomain, Role, Task, Capability, CapabilityLevel, Skill};
use crate::database::connection;
use crate::graphql::loaders::{TaskLoader, RoleLoader};

/// Data structure for a relationship between a person and work
/// This is a many to many relationship as multiple people may be
/// assigned to a specific piece of work and a person may be assigned
/// to multiple pieces of work
/// Work may be planned under a task with its capability requirement
/// (domain and capability_level) before a role is assigned
#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Insertable, AsChangeset, SimpleObject)]
#[graphql(complex)]
// treat_none_as_null: `update()` always loads the full row first and only
// overwrites the fields the caller changed, so a `None` here always means
// "this column should be NULL" (e.g. clearing blocked context on unblock),
// never "skip this column". Without this, Diesel's default AsChangeset skips
// None fields and the clears would silently not persist.
#[diesel(table_name = works, treat_none_as_null = true)]
pub struct Work {
    pub id: Uuid,
    #[graphql(skip)]
    pub task_id: Uuid,
    #[graphql(skip)]
    pub role_id: Option<Uuid>,
    pub work_description: String,
    pub url: Option<String>,
    pub domain: SkillDomain,
    pub capability_level: CapabilityLevel,
    pub effort: i32,
    pub work_status: WorkStatus,
    pub priority: Priority,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    #[graphql(skip)]
    pub skill_id: Uuid,
    /// Target completion date for this work item (Proposal 1). User-set.
    pub due_date: Option<NaiveDateTime>,
    /// Stamped server-side when the work first enters IN_PROGRESS.
    pub started_at: Option<NaiveDateTime>,
    /// Stamped server-side when the work is marked COMPLETED.
    pub completed_at: Option<NaiveDateTime>,
    /// Free-text reason the work is BLOCKED (Proposal 2).
    pub blocked_reason: Option<String>,
    /// Stamped server-side when the work enters BLOCKED, cleared when it
    /// leaves BLOCKED. Drives "blocked for N days" ageing.
    pub blocked_since: Option<NaiveDateTime>,
    /// The role/position this work is waiting on, if any — a named, reachable
    /// contact for escalation. Resolved via `blockedOnRole`.
    #[graphql(skip)]
    pub blocked_on_role_id: Option<Uuid>,
}

#[ComplexObject]
impl Work {
    pub async fn task(&self, ctx: &Context<'_>) -> Result<Task> {
        ctx.data_unchecked::<DataLoader<TaskLoader>>()
            .load_one(self.task_id)
            .await?
            .ok_or_else(|| Error::new("Task not found"))
    }

    pub async fn role(&self, ctx: &Context<'_>) -> Result<Option<Role>> {
        match self.role_id {
            Some(id) => Ok(ctx.data_unchecked::<DataLoader<RoleLoader>>().load_one(id).await?),
            None => Ok(None),
        }
    }

    /// The role/position this work is waiting on while BLOCKED, if set. Gives
    /// callers a named contact (and, through the role, a person to escalate to)
    /// rather than an opaque blocker.
    pub async fn blocked_on_role(&self, ctx: &Context<'_>) -> Result<Option<Role>> {
        match self.blocked_on_role_id {
            Some(id) => Ok(ctx.data_unchecked::<DataLoader<RoleLoader>>().load_one(id).await?),
            None => Ok(None),
        }
    }

    /// The specific skill this work requires
    pub async fn skill(&self) -> Result<Skill> {
        Skill::get_by_id(&self.skill_id)
    }

    /// Capabilities (and through them, people) validated at or above
    /// the level required by this work, ordered by validated level.
    /// Matches on the work's required skill. Accepts an optional count
    /// (default 10).
    pub async fn capability_matches(&self, count: Option<i64>) -> Result<Vec<Capability>> {
        let count = count.unwrap_or(10);

        Capability::get_matches_by_skill_id_and_level(&self.skill_id, self.capability_level, count)
    }

    /// Full status-transition history for this work, most recent first
    /// (Proposal 4). Empty for work created before status history existed.
    pub async fn status_history(&self) -> Result<Vec<WorkStatusChange>> {
        WorkStatusChange::get_by_work_id(&self.id)
    }

    /// Comments and flags on this work, most recent first (Proposal 3).
    pub async fn updates(&self) -> Result<Vec<WorkUpdate>> {
        WorkUpdate::get_by_work_id(&self.id)
    }

    /// Number of unresolved FLAG updates — the "needs attention" count a
    /// manager triages (Proposal 3).
    pub async fn open_flag_count(&self) -> Result<i64> {
        WorkUpdate::open_flag_count(&self.id)
    }
}


// Non Graphql
impl Work {
    pub fn create(work: &NewWork) -> Result<Work> {
        let mut conn = connection()?;

        let res = diesel::insert_into(works::table)
            .values(work)
            .get_result(&mut conn)?;
        
        Ok(res)
    }

    pub fn batch_create(works: &Vec<NewWork>) -> Result<usize> {
        let mut conn = connection()?;

        let res = diesel::insert_into(works::table)
            .values(works)
            .execute(&mut conn)?;
        
        Ok(res)
    }
    
    pub fn get_or_create(work: &NewWork) -> Result<Work> {
        let mut conn = connection()?;

        let res = works::table
        .filter(works::task_id.eq(&work.task_id)
            .and(works::role_id.eq(&work.role_id))
            .and(works::work_description.eq(&work.work_description)))
        .distinct()
        .first(&mut conn);
        
        let work = match res {
            Ok(p) => p,
            Err(e) => {
                // Work not found
                println!("{:?}", e);
                let p = Work::create(work).expect("Unable to create work");
                p
            }
        };
        Ok(work)
    }

    pub fn get_all() -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let persons = works::table.load::<Work>(&mut conn)?;
        Ok(persons)
    }

    /// Server-side filtered + paginated work list. Optionally narrows to a
    /// single `status` and/or to unassigned work (`role_id IS NULL`). Ordered
    /// by `created_at DESC` so pagination is stable. A `None` limit returns
    /// every matching row (preserving the old "fetch all" behaviour).
    pub fn get_filtered(status: Option<WorkStatus>, unassigned_only: bool, limit: Option<i64>, offset: i64) -> Result<Vec<Self>> {
        let mut conn = connection()?;

        let mut query = works::table.into_boxed();
        if let Some(s) = status {
            query = query.filter(works::work_status.eq(s));
        }
        if unassigned_only {
            query = query.filter(works::role_id.is_null());
        }
        query = query.order_by(works::created_at.desc());
        if let Some(l) = limit {
            query = query.limit(l).offset(offset);
        }

        let res = query.load::<Work>(&mut conn)?;
        Ok(res)
    }

    /// Total number of work items matching the same filters as `get_filtered`,
    /// ignoring limit/offset — for driving pagination controls.
    pub fn count_filtered(status: Option<WorkStatus>, unassigned_only: bool) -> Result<i64> {
        let mut conn = connection()?;

        let mut query = works::table.into_boxed();
        if let Some(s) = status {
            query = query.filter(works::work_status.eq(s));
        }
        if unassigned_only {
            query = query.filter(works::role_id.is_null());
        }

        let total = query.count().get_result(&mut conn)?;
        Ok(total)
    }

    pub fn get_count(count: i64) -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let persons = works::table
            .limit(count)
            .load::<Work>(&mut conn)?;
        Ok(persons)
    }

    pub fn get_worker_ids(task_id: &Uuid) -> Result<Vec<Uuid>> {

        let mut conn = connection()?;
        let res: Vec<Option<Uuid>> = works::table
            .filter(works::task_id.eq(task_id))
            .select(works::role_id)
            .load::<Option<Uuid>>(&mut conn)?;

        Ok(res.into_iter().flatten().collect())
    }

    pub fn get_by_id(id: &Uuid) -> Result<Self> {
        let mut conn = connection()?;
        let person = works::table
            .filter(works::id.eq(id))
            .first(&mut conn)?;
        Ok(person)
    }

    pub fn get_by_role_id(role_id: &Uuid) -> Result<Vec<Self>> {
        let mut conn = connection()?;

        let res = works::table
            .filter(works::role_id.eq(role_id))
            .order_by(works::created_at)
            .load::<Work>(&mut conn)?;

        Ok(res)
    }

    /// Batched lookup for the DataLoader: fetch the work for many roles in one
    /// query. Returns a flat list ordered by role then creation; the loader
    /// groups it by `role_id`.
    pub fn get_by_role_ids(role_ids: &[Uuid]) -> Result<Vec<Self>> {
        let mut conn = connection()?;

        let res = works::table
            .filter(works::role_id.eq_any(role_ids))
            .order_by((works::role_id, works::created_at))
            .load::<Work>(&mut conn)?;

        Ok(res)
    }

    /// Return the numeric indicator of the total effort allocated to a person.
    pub fn sum_role_effort(role_id: &Uuid) -> Result<i32> {
        let mut conn = connection()?;

        let res = works::table
            .filter(works::role_id.eq(role_id))
            .filter(works::work_status.ne_all(vec![WorkStatus::Cancelled, WorkStatus::Completed]))
            .select(works::effort)
            .load::<i32>(&mut conn)?;

        let total_effort = res.into_iter()
            .sum();

        Ok(total_effort)
    }

    /// Return the total effort of active work across a person's active roles.
    pub fn sum_person_active_effort(person_id: &Uuid) -> Result<i32> {
        let mut conn = connection()?;

        let res = works::table
            .inner_join(roles::table)
            .filter(roles::person_id.eq(person_id))
            .filter(roles::active.eq(true))
            .filter(works::work_status.ne_all(vec![WorkStatus::Cancelled, WorkStatus::Completed]))
            .select(works::effort)
            .load::<i32>(&mut conn)?;

        let total_effort = res.into_iter()
            .sum();

        Ok(total_effort)
    }

    /// Return the numeric indicator of the total effort allocated to a task.
    pub fn sum_task_effort(task_id: &Uuid) -> Result<i32> {
        let mut conn = connection()?;

        let res = works::table
            .filter(works::task_id.eq(task_id))
            .select(works::effort)
            .load::<i32>(&mut conn)?;

        let total_effort = res.into_iter()
            .sum();

        Ok(total_effort)
    }

    pub fn get_by_task_id(task_id: &Uuid) -> Result<Vec<Self>> {
        let mut conn = connection()?;

        let res = works::table
            .filter(works::task_id.eq(task_id))
            .load::<Work>(&mut conn)?;

        Ok(res)
    }

    /// Return work under a task that has not yet been assigned to a role
    pub fn get_vacant_by_task_id(task_id: &Uuid) -> Result<Vec<Self>> {
        let mut conn = connection()?;

        let res = works::table
            .filter(works::task_id.eq(task_id))
            .filter(works::role_id.is_null())
            .order_by(works::created_at)
            .load::<Work>(&mut conn)?;

        Ok(res)
    }

    /// Return all work under a product's tasks
    pub fn get_by_product_id(product_id: &Uuid) -> Result<Vec<Self>> {
        let mut conn = connection()?;

        let res = works::table
            .inner_join(tasks::table)
            .filter(tasks::product_id.eq(product_id))
            .select(works::all_columns)
            .order_by(works::created_at)
            .load::<Work>(&mut conn)?;

        Ok(res)
    }

    /// Return work under a product's tasks that has not yet been assigned to a role
    pub fn get_vacant_by_product_id(product_id: &Uuid) -> Result<Vec<Self>> {
        let mut conn = connection()?;

        let res = works::table
            .inner_join(tasks::table)
            .filter(tasks::product_id.eq(product_id))
            .filter(works::role_id.is_null())
            .select(works::all_columns)
            .order_by(works::created_at)
            .load::<Work>(&mut conn)?;

        Ok(res)
    }

    /// Return the numeric indicator of the total effort allocated to a product's tasks.
    pub fn sum_product_effort(product_id: &Uuid) -> Result<i32> {
        let mut conn = connection()?;

        let res = works::table
            .inner_join(tasks::table)
            .filter(tasks::product_id.eq(product_id))
            .filter(works::work_status.ne_all(vec![WorkStatus::Cancelled, WorkStatus::Completed]))
            .select(works::effort)
            .load::<i32>(&mut conn)?;

        let total_effort = res.into_iter()
            .sum();

        Ok(total_effort)
    }

    pub fn update(&self) -> Result<Self> {
        let mut conn = connection()?;

        let res = diesel::update(works::table)
        .filter(works::id.eq(&self.id))
        .set(self)
        .get_result(&mut conn)?;

        Ok(res)
    }

    /// Maintain the server-managed lifecycle timestamps when the work's status
    /// changes from `old_status` to its current `work_status`. Idempotent for
    /// no-op transitions (call only guards on an actual change). See Proposals
    /// 1 and 2:
    ///  - first entry to IN_PROGRESS stamps `started_at`
    ///  - entry to COMPLETED stamps `completed_at` (and backfills `started_at`)
    ///  - entry to BLOCKED stamps `blocked_since`; leaving BLOCKED clears it
    pub fn apply_status_transition(&mut self, old_status: WorkStatus) {
        if self.work_status == old_status {
            return;
        }
        let now = Utc::now().naive_utc();
        match self.work_status {
            WorkStatus::InProgress => {
                if self.started_at.is_none() {
                    self.started_at = Some(now);
                }
            }
            WorkStatus::Completed => {
                self.completed_at = Some(now);
                if self.started_at.is_none() {
                    self.started_at = Some(now);
                }
            }
            WorkStatus::Blocked => {
                self.blocked_since = Some(now);
            }
            _ => {}
        }
        if old_status == WorkStatus::Blocked && self.work_status != WorkStatus::Blocked {
            self.blocked_since = None;
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Insertable, InputObject)]
#[diesel(table_name = works)]
pub struct NewWork {
    pub task_id: Uuid,
    pub role_id: Option<Uuid>,
    pub work_description: String,
    pub url: Option<String>,
    pub domain: SkillDomain,
    pub skill_id: Uuid,
    pub capability_level: CapabilityLevel,
    pub effort: i32,
    pub work_status: WorkStatus,
    pub priority: Priority,
    /// Optional target completion date set at creation time (Proposal 1).
    pub due_date: Option<NaiveDateTime>,
}

impl NewWork {

    pub fn new(
        task_id: Uuid,
        role_id: Option<Uuid>,
        work_description: String,
        url: Option<String>,
        domain: SkillDomain,
        skill_id: Uuid,
        capability_level: CapabilityLevel,
        effort: i32,
        work_status: WorkStatus,
        priority: Priority,
        due_date: Option<NaiveDateTime>,
    ) -> Self {
        NewWork {
            task_id,
            role_id,
            work_description,
            url,
            domain,
            skill_id,
            capability_level,
            effort,
            work_status,
            priority,
            due_date,
        }
    }
}

/// A single recorded work_status transition (Proposal 4). `from_status` is
/// None for the row logged when the work was created.
#[derive(Debug, Clone, Deserialize, Serialize, Queryable, SimpleObject)]
#[graphql(complex)]
#[diesel(table_name = work_status_history)]
pub struct WorkStatusChange {
    pub id: Uuid,
    #[graphql(skip)]
    pub work_id: Uuid,
    pub from_status: Option<WorkStatus>,
    pub to_status: WorkStatus,
    pub changed_at: NaiveDateTime,
    #[graphql(skip)]
    pub changed_by_user_id: Option<Uuid>,
}

#[ComplexObject]
impl WorkStatusChange {
    /// Display name of the user who made the change, if still known.
    pub async fn changed_by_name(&self) -> Option<String> {
        self.changed_by_user_id
            .and_then(|id| crate::models::User::get_by_id(&id).ok().map(|u| u.name))
    }
}

impl WorkStatusChange {
    pub fn get_by_work_id(work_id: &Uuid) -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let res = work_status_history::table
            .filter(work_status_history::work_id.eq(work_id))
            .order_by(work_status_history::changed_at.desc())
            .load::<WorkStatusChange>(&mut conn)?;
        Ok(res)
    }

    /// Append a transition record. Best-effort: status logging must never fail
    /// the surrounding mutation, so errors are swallowed.
    pub fn record(
        work_id: Uuid,
        from_status: Option<WorkStatus>,
        to_status: WorkStatus,
        changed_by_user_id: Option<Uuid>,
    ) {
        if let Ok(mut conn) = connection() {
            let new = NewWorkStatusChange { work_id, from_status, to_status, changed_by_user_id };
            let _ = diesel::insert_into(work_status_history::table)
                .values(&new)
                .execute(&mut conn);
        }
    }
}

#[derive(Debug, Insertable)]
#[diesel(table_name = work_status_history)]
struct NewWorkStatusChange {
    work_id: Uuid,
    from_status: Option<WorkStatus>,
    to_status: WorkStatus,
    changed_by_user_id: Option<Uuid>,
}

/// Kind of a work update (Proposal 3): a plain comment, or a flag raised for
/// management attention.
#[derive(Debug, Clone, Copy, PartialEq, Eq, DbEnum, Serialize, Deserialize, Enum)]
#[ExistingTypePath = "crate::schema::sql_types::WorkUpdateKind"]
pub enum WorkUpdateKind {
    Comment,
    Flag,
}

/// A comment or flag left on a work item (Proposal 3). Only FLAG rows use the
/// resolve columns; a flag with `flag_resolved_at = NULL` is open.
#[derive(Debug, Clone, Deserialize, Serialize, Queryable, SimpleObject)]
#[graphql(complex)]
#[diesel(table_name = work_updates)]
pub struct WorkUpdate {
    pub id: Uuid,
    #[graphql(skip)]
    pub work_id: Uuid,
    #[graphql(skip)]
    pub author_user_id: Option<Uuid>,
    pub kind: WorkUpdateKind,
    pub body: String,
    pub created_at: NaiveDateTime,
    pub flag_resolved_at: Option<NaiveDateTime>,
    #[graphql(skip)]
    pub resolved_by_user_id: Option<Uuid>,
}

#[ComplexObject]
impl WorkUpdate {
    /// Display name of the author, if still known.
    pub async fn author_name(&self) -> Option<String> {
        self.author_user_id
            .and_then(|id| crate::models::User::get_by_id(&id).ok().map(|u| u.name))
    }

    /// Display name of whoever resolved the flag, if any.
    pub async fn resolved_by_name(&self) -> Option<String> {
        self.resolved_by_user_id
            .and_then(|id| crate::models::User::get_by_id(&id).ok().map(|u| u.name))
    }
}

impl WorkUpdate {
    pub fn get_by_id(id: &Uuid) -> Result<Self> {
        let mut conn = connection()?;
        Ok(work_updates::table.filter(work_updates::id.eq(id)).first(&mut conn)?)
    }

    pub fn get_by_work_id(work_id: &Uuid) -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let res = work_updates::table
            .filter(work_updates::work_id.eq(work_id))
            .order_by(work_updates::created_at.desc())
            .load::<WorkUpdate>(&mut conn)?;
        Ok(res)
    }

    pub fn open_flag_count(work_id: &Uuid) -> Result<i64> {
        let mut conn = connection()?;
        let n = work_updates::table
            .filter(work_updates::work_id.eq(work_id))
            .filter(work_updates::kind.eq(WorkUpdateKind::Flag))
            .filter(work_updates::flag_resolved_at.is_null())
            .count()
            .get_result(&mut conn)?;
        Ok(n)
    }

    /// All unresolved flags across every work item, newest first (capped). The
    /// caller is responsible for scoping these to what the principal manages —
    /// this is the raw feed behind the manager flags queue.
    pub fn open_flags(limit: i64) -> Result<Vec<Self>> {
        let mut conn = connection()?;
        let res = work_updates::table
            .filter(work_updates::kind.eq(WorkUpdateKind::Flag))
            .filter(work_updates::flag_resolved_at.is_null())
            .order_by(work_updates::created_at.desc())
            .limit(limit)
            .load::<WorkUpdate>(&mut conn)?;
        Ok(res)
    }

    pub fn create(
        work_id: Uuid,
        author_user_id: Option<Uuid>,
        kind: WorkUpdateKind,
        body: String,
    ) -> Result<Self> {
        let mut conn = connection()?;
        let new = NewWorkUpdate { work_id, author_user_id, kind, body };
        let res = diesel::insert_into(work_updates::table)
            .values(&new)
            .get_result(&mut conn)?;
        Ok(res)
    }

    /// Mark an open flag resolved. No-op fields for comments; safe to call on
    /// an already-resolved flag (it just refreshes the resolver/time).
    pub fn resolve_flag(id: &Uuid, resolved_by_user_id: Option<Uuid>) -> Result<Self> {
        let mut conn = connection()?;
        let res = diesel::update(work_updates::table.filter(work_updates::id.eq(id)))
            .set((
                work_updates::flag_resolved_at.eq(Some(Utc::now().naive_utc())),
                work_updates::resolved_by_user_id.eq(resolved_by_user_id),
            ))
            .get_result(&mut conn)?;
        Ok(res)
    }
}

#[derive(Debug, Insertable)]
#[diesel(table_name = work_updates)]
struct NewWorkUpdate {
    work_id: Uuid,
    author_user_id: Option<Uuid>,
    kind: WorkUpdateKind,
    body: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, DbEnum, Serialize, Deserialize, Enum)]
#[ExistingTypePath = "crate::schema::sql_types::WorkStatus"]
pub enum WorkStatus {
    Planning,
    InProgress,
    Completed,
    Blocked,
    Cancelled,
}

impl Distribution<WorkStatus> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> WorkStatus {
        match rng.gen_range(0..=10) {
            0..=1 => WorkStatus::Planning,
            2..=7 => WorkStatus::InProgress,
            8 => WorkStatus::Completed,
            9 => WorkStatus::Cancelled,
            10 => WorkStatus::Blocked,
            _ => WorkStatus::Blocked,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, DbEnum, Serialize, Deserialize, Enum)]
#[ExistingTypePath = "crate::schema::sql_types::Priority"]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl Distribution<Priority> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Priority {
        match rng.gen_range(0..=3) {
            0 => Priority::Low,
            1 => Priority::Medium,
            2 => Priority::High,
            _ => Priority::Critical,
        }
    }
}
