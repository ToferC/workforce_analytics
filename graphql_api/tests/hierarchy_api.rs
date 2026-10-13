//! End-to-end checks for the hierarchy list filters, roll-ups, ancestry and
//! person job matches, run through the real schema and loaders against a
//! scratch Postgres database (migrations are applied on first use):
//!
//!   DATABASE_URL=postgres://… cargo test -p graphql_api --test hierarchy_api -- --ignored
//!
//! Each test builds its own organization, so the database may be shared, and
//! a lock serializes the tests so the SQL statement counter sees one request
//! at a time.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{LazyLock, Once};

use async_graphql::Request;
use chrono::{SubsecRound, Utc};
use diesel::connection::{Instrumentation, InstrumentationEvent, set_default_instrumentation};
use diesel::prelude::*;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use futures::lock::{Mutex, MutexGuard};
use serde_json::{Value, json};
use uuid::Uuid;

use graphql_api::common_utils::UserRole;
use graphql_api::database::{POOL, connection};
use graphql_api::graphql::{create_schema_with_context, with_loaders};
use graphql_api::models::*;
use graphql_api::schema::{capabilities, teams};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../migrations");

static SETUP: Once = Once::new();
static SERIAL: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));
static STATEMENTS: AtomicUsize = AtomicUsize::new(0);

/// Counts SQL statements, except connection bookkeeping that varies with
/// which pooled connection serves a query: r2d2's `SELECT 1` check and
/// Diesel's once-per-connection enum type lookups.
fn count_statements() -> Option<Box<dyn Instrumentation>> {
    Some(Box::new(|event: InstrumentationEvent<'_>| {
        if let InstrumentationEvent::StartQuery { query, .. } = event {
            let sql = query.to_string();
            if !sql.starts_with("SELECT 1 ") && !sql.contains(r#""pg_type""#) {
                STATEMENTS.fetch_add(1, Ordering::SeqCst);
            }
        }
    }))
}

async fn setup() -> MutexGuard<'static, ()> {
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

/// Execute a query as an admin with per-request loaders; returns the data and
/// the number of SQL statements it ran.
async fn run(query: &str) -> (Value, usize) {
    let schema = create_schema_with_context(POOL.clone());
    let before = STATEMENTS.load(Ordering::SeqCst);
    let res = schema
        .execute(with_loaders(Request::new(query)).data(UserRole::Admin))
        .await;
    assert!(res.errors.is_empty(), "{query}: {:?}", res.errors);
    (
        res.data.into_json().unwrap(),
        STATEMENTS.load(Ordering::SeqCst) - before,
    )
}

fn org() -> Uuid {
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

fn tier(org: Uuid, parent: Option<Uuid>) -> Uuid {
    let name = Uuid::new_v4().to_string();
    let t = NewOrgTier::new(org, 0, name.clone(), name, SkillDomain::Governance, parent);
    OrgTier::create(&t).unwrap().id
}

fn team(org: Uuid, tier: Uuid, name: &str) -> Uuid {
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

fn role(team: Uuid, person: Option<Uuid>, active: bool, title: &str) -> Uuid {
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

fn person(org: Uuid) -> Uuid {
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
        personnel_type: PersonnelType::Civilian,
    })
    .unwrap()
    .id
}

fn ids(list: &Value) -> Vec<&str> {
    list.as_array()
        .unwrap()
        .iter()
        .map(|x| x["id"].as_str().unwrap())
        .collect()
}

#[actix_rt::test]
#[ignore = "needs DATABASE_URL"]
async fn tier_and_organization_roll_up_vacancies_over_the_whole_subtree() {
    let _serial = setup().await;
    let org = org();
    let root = tier(org, None);
    let child = tier(org, Some(root));
    let grandchild = tier(org, Some(child));
    let (root_team, deep_team) = (team(org, root, "Root"), team(org, grandchild, "Deep"));
    let alice = person(org);
    role(root_team, None, true, "Vacant");
    role(root_team, Some(alice), true, "Filled");
    role(deep_team, None, true, "Vacant deep");
    role(deep_team, None, true, "Vacant deep 2");
    role(deep_team, None, false, "Ended");

    let (data, _) = run(&format!(
        r#"{{ organizationById(id: "{org}") {{ headcount totalEffort vacantRoleCount }}
             root: orgTierById(id: "{root}") {{ vacantRoleCount headcount }}
             child: orgTierById(id: "{child}") {{ vacantRoleCount headcount }}
             teamByID(id: "{deep_team}") {{ vacantRoleCount }} }}"#
    ))
    .await;

    assert_eq!(
        data["organizationById"],
        json!({"headcount": 1, "totalEffort": 0, "vacantRoleCount": 3})
    );
    assert_eq!(data["root"], json!({"vacantRoleCount": 3, "headcount": 1}));
    assert_eq!(data["child"], json!({"vacantRoleCount": 2, "headcount": 0}));
    assert_eq!(data["teamByID"]["vacantRoleCount"], 2);
}

#[actix_rt::test]
#[ignore = "needs DATABASE_URL"]
async fn list_roll_ups_run_a_constant_number_of_statements() {
    let _serial = setup().await;
    let statements = |org: Uuid| async move {
        let query = format!(
            r#"{{ allTeams(organizationId: "{org}") {{ headcount totalEffort vacantRoleCount }}
                 orgTiersByOrgId(id: "{org}") {{ headcount totalEffort vacantRoleCount }} }}"#
        );
        run(&query).await.1
    };

    let small = org();
    let t = tier(small, None);
    role(team(small, t, "Only"), None, true, "Vacant");

    let large = org();
    let root = tier(large, None);
    for i in 0..6 {
        let t = tier(large, Some(root));
        role(team(large, t, &format!("Team {i}")), None, true, "Vacant");
    }

    assert_eq!(statements(small).await, statements(large).await);
}

#[actix_rt::test]
#[ignore = "needs DATABASE_URL"]
async fn team_filters_combine_with_search_and_pagination() {
    let _serial = setup().await;
    let (org, other) = (org(), org());
    let (root, child) = (tier(org, None), tier(org, None));
    let alpha = [
        team(org, root, "Alpha one"),
        team(org, root, "Alpha two"),
        team(org, root, "Alpha three"),
    ];
    team(org, root, "Beta");
    team(org, child, "Alpha child");
    team(other, tier(other, None), "Alpha elsewhere");
    diesel::update(teams::table.find(alpha[2]))
        .set(teams::retired_at.eq(Utc::now().naive_utc()))
        .execute(&mut connection().unwrap())
        .unwrap();

    let (data, _) = run(&format!(
        r#"{{ page: allTeams(organizationId: "{org}", orgTierId: "{root}", search: "alpha", limit: 1, offset: 1) {{ nameEnglish }}
             total: teamsCount(organizationId: "{org}", orgTierId: "{root}", search: "alpha")
             withRetired: teamsCount(organizationId: "{org}", orgTierId: "{root}", search: "alpha", includeRetired: true)
             orgWide: teamsCount(organizationId: "{org}", search: "alpha") }}"#
    ))
    .await;

    // Active "Alpha" teams on the root tier, by name: "Alpha one", "Alpha two".
    assert_eq!(data["page"], json!([{"nameEnglish": "Alpha two"}]));
    assert_eq!(data["total"], 2);
    assert_eq!(data["withRetired"], 3);
    assert_eq!(data["orgWide"], 3);
}

#[actix_rt::test]
#[ignore = "needs DATABASE_URL"]
async fn roles_include_ended_only_when_asked() {
    let _serial = setup().await;
    let org = org();
    let t = team(org, tier(org, None), "Team");
    let tag = Uuid::new_v4().simple().to_string();
    let active = [
        role(t, None, true, &format!("{tag} a")),
        role(t, None, true, &format!("{tag} b")),
    ];
    let ended = role(t, None, false, &format!("{tag} c"));

    let (data, _) = run(&format!(
        r#"{{ current: allRoles(organizationId: "{org}", search: "{tag}") {{ id }}
             currentCount: rolesCount(organizationId: "{org}", search: "{tag}")
             lastPage: allRoles(organizationId: "{org}", search: "{tag}", includeEnded: true, limit: 2, offset: 2) {{ id }}
             allCount: rolesCount(organizationId: "{org}", search: "{tag}", includeEnded: true) }}"#
    ))
    .await;

    assert_eq!(
        ids(&data["current"]),
        active.iter().map(Uuid::to_string).collect::<Vec<_>>()
    );
    assert_eq!(data["currentCount"], 2);
    assert_eq!(ids(&data["lastPage"]), vec![ended.to_string()]);
    assert_eq!(data["allCount"], 3);
}

#[actix_rt::test]
#[ignore = "needs DATABASE_URL"]
async fn ancestors_run_from_root_to_parent() {
    let _serial = setup().await;
    let org = org();
    let root = tier(org, None);
    let middle = tier(org, Some(root));
    let leaf = tier(org, Some(middle));

    let (data, _) = run(&format!(
        r#"{{ leaf: orgTierById(id: "{leaf}") {{ ancestors {{ id }} }}
             root: orgTierById(id: "{root}") {{ ancestors {{ id }} }} }}"#
    ))
    .await;

    assert_eq!(
        ids(&data["leaf"]["ancestors"]),
        vec![root.to_string(), middle.to_string()]
    );
    assert_eq!(data["root"]["ancestors"], json!([]));
}

#[actix_rt::test]
#[ignore = "needs DATABASE_URL"]
async fn team_retired_at_is_null_until_retired() {
    let _serial = setup().await;
    let org = org();
    let t = team(org, tier(org, None), "Team");
    let query = format!(r#"{{ teamByID(id: "{t}") {{ retiredAt }} }}"#);

    assert_eq!(run(&query).await.0["teamByID"]["retiredAt"], Value::Null);

    // Postgres keeps microseconds.
    let retired = Utc::now().naive_utc().trunc_subsecs(6);
    diesel::update(teams::table.find(t))
        .set(teams::retired_at.eq(retired))
        .execute(&mut connection().unwrap())
        .unwrap();
    let returned = run(&query).await.0["teamByID"]["retiredAt"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(returned.parse::<chrono::NaiveDateTime>().unwrap(), retired);
}

#[actix_rt::test]
#[ignore = "needs DATABASE_URL"]
async fn person_fuzzy_matches_rank_by_score_using_validated_levels() {
    let _serial = setup().await;
    let org = org();
    let t = team(org, tier(org, None), "Team");
    let tag = Uuid::new_v4().to_string();
    let skill = |name: &str| {
        Skill::create(&NewSkill::new(
            format!("{name} {tag}"),
            format!("{name} {tag}"),
            SkillDomain::Governance,
            None,
            None,
        ))
        .unwrap()
        .id
    };
    let (a, b) = (skill("A"), skill("B"));
    let candidate = person(org);

    // A: self-identified Specialist, validated Experienced. B: unvalidated Expert.
    let cap_a = Capability::create(&NewCapability::new(
        candidate,
        a,
        org,
        CapabilityLevel::Specialist,
    ))
    .unwrap();
    diesel::update(capabilities::table.find(cap_a.id))
        .set(capabilities::validated_level.eq(Some(CapabilityLevel::Experienced)))
        .execute(&mut connection().unwrap())
        .unwrap();
    Capability::create(&NewCapability::new(
        candidate,
        b,
        org,
        CapabilityLevel::Expert,
    ))
    .unwrap();

    let with_requirements = |reqs: &[(Uuid, CapabilityLevel)]| {
        let r = role(t, None, true, "Open");
        for &(skill, level) in reqs {
            Requirement::create(&NewRequirement::new(r, skill, level)).unwrap();
        }
        r
    };
    let exact = with_requirements(&[
        (a, CapabilityLevel::Experienced),
        (b, CapabilityLevel::Expert),
    ]);
    let one_short =
        with_requirements(&[(a, CapabilityLevel::Expert), (b, CapabilityLevel::Novice)]);
    let too_far = with_requirements(&[(a, CapabilityLevel::Specialist)]);

    let (data, _) = run(&format!(
        r#"{{ personById(id: "{candidate}") {{ fuzzyMatches(limit: 1000) {{
               role {{ id }} matchScore requirementGaps {{ skillId actualLevel }} }} }} }}"#
    ))
    .await;

    // Other tests' open roles may match too; keep only this test's.
    let ours: Vec<&Value> = data["personById"]["fuzzyMatches"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| {
            [exact, one_short, too_far]
                .iter()
                .any(|r| m["role"]["id"] == r.to_string())
        })
        .collect();
    assert_eq!(
        ours.iter()
            .map(|m| m["role"]["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec![exact.to_string(), one_short.to_string()]
    );
    assert_eq!(ours[0]["matchScore"], 1.0);
    let gap_a = ours[1]["requirementGaps"]
        .as_array()
        .unwrap()
        .iter()
        .find(|g| g["skillId"] == a.to_string())
        .unwrap();
    assert_eq!(gap_a["actualLevel"], "EXPERIENCED");
}
