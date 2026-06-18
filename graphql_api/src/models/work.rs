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

use crate::schema::*;
use crate::models::{SkillDomain, Role, Task, Capability, CapabilityLevel, Skill};
use crate::database::connection;

/// Data structure for a relationship between a person and work
/// This is a many to many relationship as multiple people may be
/// assigned to a specific piece of work and a person may be assigned
/// to multiple pieces of work
/// Work may be planned under a task with its capability requirement
/// (domain and capability_level) before a role is assigned
#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Insertable, AsChangeset, SimpleObject)]
#[graphql(complex)]
#[table_name = "works"]
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
}

#[ComplexObject]
impl Work {
    pub async fn task(&self) -> Result<Task> {
        Task::get_by_id(&self.task_id)
    }

    pub async fn role(&self) -> Result<Option<Role>> {
        match self.role_id {
            Some(id) => Ok(Some(Role::get_by_id(&id)?)),
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
}

#[derive(Debug, Clone, Deserialize, Serialize, Insertable, InputObject)]
#[table_name = "works"]
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
        }
    }
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
