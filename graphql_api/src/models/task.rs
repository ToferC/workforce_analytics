use std::fmt::Debug;

use chrono::{prelude::*};
use serde::{Deserialize, Serialize};
use diesel::{self, Insertable, Queryable, ExpressionMethods, BoolExpressionMethods, PgTextExpressionMethods};
use diesel::{RunQueryDsl, QueryDsl};
use uuid::Uuid;
use async_graphql::*;

use crate::schema::*;
use crate::database::connection;

use crate::models::{Priority, SkillDomain, WorkStatus};

use super::{Work, Role, Product};

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Insertable, AsChangeset, SimpleObject)]
#[graphql(complex)]
#[table_name = "tasks"]
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
    pub async fn product(&self) -> Result<Option<Product>> {
        match self.product_id {
            Some(id) => Ok(Some(Product::get_by_id(&id)?)),
            None => Ok(None),
        }
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
}

#[derive(Debug, Clone, Deserialize, Serialize, Insertable, SimpleObject, InputObject)]
#[table_name = "tasks"]
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
