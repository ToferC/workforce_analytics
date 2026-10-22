//! Shared harness for the DB-backed API tests: migrations on first use, a
//! lock that runs one test at a time, a SQL statement counter, and fixtures
//! that build a fresh organization per test.
#![allow(dead_code)]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{LazyLock, Once};

use async_graphql::Request;
use chrono::Utc;
use diesel::connection::{Instrumentation, InstrumentationEvent, set_default_instrumentation};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use futures::lock::{Mutex, MutexGuard};
use serde_json::Value;
use uuid::Uuid;

use graphql_api::common_utils::UserRole;
use graphql_api::database::{POOL, connection};
use graphql_api::graphql::{create_schema_with_context, with_loaders};
use graphql_api::models::*;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../migrations");

pub static SETUP: Once = Once::new();
pub static SERIAL: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));
pub static STATEMENTS: AtomicUsize = AtomicUsize::new(0);

/// Counts SQL statements, except connection bookkeeping that varies with
/// which pooled connection serves a query: r2d2's `SELECT 1` check and
/// Diesel's once-per-connection enum type lookups.
pub fn count_statements() -> Option<Box<dyn Instrumentation>> {
    Some(Box::new(|event: InstrumentationEvent<'_>| {
        if let InstrumentationEvent::StartQuery { query, .. } = event {
            let sql = query.to_string();
            if !sql.starts_with("SELECT 1 ") && !sql.contains(r#""pg_type""#) {
                STATEMENTS.fetch_add(1, Ordering::SeqCst);
            }
        }
    }))
}

pub async fn setup() -> MutexGuard<'static, ()> {
    let guard = SERIAL.lock().await;
    SETUP.call_once(|| {
        // Must precede the pool's first connection to instrument it.
        set_default_instrumentation(count_statements).expect("instrumentation");
        connection()
            .expect("DATABASE_URL")
            .run_pending_migrations(MIGRATIONS)
            .expect("migrations");
    });
    guard
}

/// Execute a query or mutation as an admin with per-request loaders.
pub async fn execute(query: &str) -> async_graphql::Response {
    let schema = create_schema_with_context(POOL.clone());
    schema
        .execute(with_loaders(Request::new(query)).data(UserRole::Admin))
        .await
}

/// Like `execute`, asserting success; returns the data and the number of SQL
/// statements it ran.
pub async fn run(query: &str) -> (Value, usize) {
    let before = STATEMENTS.load(Ordering::SeqCst);
    let res = execute(query).await;
    assert!(res.errors.is_empty(), "{query}: {:?}", res.errors);
    (
        res.data.into_json().unwrap(),
        STATEMENTS.load(Ordering::SeqCst) - before,
    )
}

pub fn org() -> Uuid {
    let tag = Uuid::new_v4().simple().to_string();
    let acronym = tag[..16].to_string();
    let org = NewOrganization::new(
        tag.clone(),
        tag,
        acronym.clone(),
        acronym,
        "test".into(),
        "".into(),
    );
    Organization::create(&org).unwrap().id
}

pub fn tier(org: Uuid, parent: Option<Uuid>) -> Uuid {
    let name = Uuid::new_v4().to_string();
    let t = NewOrgTier::new(org, 0, name.clone(), name, SkillDomain::Governance, parent);
    OrgTier::create(&t).unwrap().id
}

pub fn team(org: Uuid, tier: Uuid, name: &str) -> Uuid {
    let t = NewTeam::new(
        name.into(),
        name.into(),
        org,
        tier,
        SkillDomain::Governance,
        "".into(),
        "".into(),
    );
    Team::create(&t).unwrap().id
}

pub fn role(team: Uuid, person: Option<Uuid>, active: bool, title: &str) -> Uuid {
    Role::create(&NewRole {
        person_id: person,
        team_id: team,
        title_en: title.into(),
        title_fr: title.into(),
        effort: 1.0,
        active,
        military_occupation: None,
        rank: None,
        occupational_group: None,
        occupational_level: None,
        start_datestamp: Utc::now().naive_utc(),
        end_date: None,
        reports_to: None,
    })
    .unwrap()
    .id
}

pub fn person(org: Uuid) -> Uuid {
    person_of(org, PersonnelType::Civilian)
}

pub fn person_of(org: Uuid, personnel_type: PersonnelType) -> Uuid {
    let tag = Uuid::new_v4().to_string();
    Person::create_with_provisioned_user(&NewPersonInput {
        family_name: tag.clone(),
        given_name: "Test".into(),
        email: format!("{tag}@example.com"),
        phone: tag[..16].into(),
        work_address: "".into(),
        city: "".into(),
        province: "".into(),
        postal_code: "".into(),
        country: "".into(),
        organization_id: org,
        peoplesoft_id: tag.clone(),
        orcid_id: tag,
        personnel_type,
    })
    .unwrap()
    .id
}

pub fn ids(list: &Value) -> Vec<&str> {
    list.as_array()
        .unwrap()
        .iter()
        .map(|x| x["id"].as_str().unwrap())
        .collect()
}
