//! What the frontend's edit forms read and write: team fields round-trip as
//! entered, and `updateRole` corrects effort and classification while keeping
//! the incumbent and reporting-line rules. Runs against a scratch database:
//!
//!   DATABASE_URL=postgres://… cargo test -p graphql_api --test forms_api -- --ignored

mod common;

use diesel::prelude::*;
use serde_json::json;
use uuid::Uuid;

use common::*;
use graphql_api::database::connection;
use graphql_api::models::*;
use graphql_api::schema::roles;

async fn update_role(id: Uuid, fields: &str) -> Result<Role, String> {
    let res = execute(&format!(
        r#"mutation {{ updateRole(roleData: {{ id: "{id}", {fields} }}) {{ id }} }}"#
    ))
    .await;
    match res.errors.first() {
        Some(e) => Err(e.message.clone()),
        None => Ok(Role::get_by_id(&id).unwrap()),
    }
}

fn classify(id: Uuid, rank: Option<Rank>, reports_to: Option<Uuid>) {
    diesel::update(roles::table.find(id))
        .set((
            roles::military_occupation.eq(rank.map(|_| MilitaryOccupation::Signals)),
            roles::rank.eq(rank),
            roles::reports_to.eq(reports_to),
        ))
        .execute(&mut connection().unwrap())
        .unwrap();
}

#[actix_rt::test]
#[ignore = "needs DATABASE_URL"]
async fn team_fields_return_what_was_saved() {
    let _serial = setup().await;
    let org = org();
    let t = NewTeam::new(
        "Cyber Team".into(),
        "Équipe cyber".into(),
        org,
        tier(org, None),
        SkillDomain::CyberSecurity,
        "Defends networks".into(),
        "Défend les réseaux".into(),
    );
    let id = Team::create(&t).unwrap().id;

    let (data, _) = run(&format!(
        r#"{{ teamByID(id: "{id}") {{ nameEnglish nameFrench descriptionEnglish descriptionFrench primaryDomain }} }}"#
    ))
    .await;

    assert_eq!(
        data["teamByID"],
        json!({
            "nameEnglish": "Cyber Team", "nameFrench": "Équipe cyber",
            "descriptionEnglish": "Defends networks", "descriptionFrench": "Défend les réseaux",
            "primaryDomain": "CYBER_SECURITY",
        })
    );
}

#[actix_rt::test]
#[ignore = "needs DATABASE_URL"]
async fn update_role_corrects_effort_and_replaces_the_classification() {
    let _serial = setup().await;
    let org = org();
    let r = role(team(org, tier(org, None), "Team"), None, true, "Analyst");

    let military = update_role(
        r,
        "effort: 2.5, classification: { militaryOccupation: SIGNALS, rank: CAPTAIN }",
    )
    .await
    .unwrap();
    assert_eq!((military.effort, military.rank), (2.5, Some(Rank::Captain)));

    // Switching stream clears the other one; omitting `classification` leaves it alone.
    let civilian = update_role(
        r,
        "classification: { occupationalGroup: COMPUTER_SYSTEMS, occupationalLevel: 3 }",
    )
    .await
    .unwrap();
    assert_eq!(
        (
            civilian.rank,
            civilian.military_occupation,
            civilian.occupational_level
        ),
        (None, None, Some(3))
    );
    let untouched = update_role(r, r#"titleEn: "Senior Analyst""#)
        .await
        .unwrap();
    assert_eq!(
        (untouched.occupational_level, untouched.effort),
        (Some(3), 2.5)
    );

    let both = update_role(r, "classification: { rank: CAPTAIN, occupationalLevel: 3 }").await;
    assert!(both.unwrap_err().contains("not both"));
    assert!(
        update_role(r, "effort: -1")
            .await
            .unwrap_err()
            .contains("negative")
    );
}

#[actix_rt::test]
#[ignore = "needs DATABASE_URL"]
async fn update_role_keeps_incumbent_and_reporting_rules() {
    let _serial = setup().await;
    let org = org();
    let t = team(org, tier(org, None), "Team");

    // A civilian incumbent needs a civilian classification.
    let held = role(
        t,
        Some(person_of(org, PersonnelType::Civilian)),
        true,
        "Held",
    );
    let err = update_role(
        held,
        "classification: { militaryOccupation: SIGNALS, rank: CAPTAIN }",
    )
    .await
    .unwrap_err();
    assert!(err.contains("civilian personnel"), "{err}");

    // A captain reporting to a major can't become a colonel, and the major
    // can't drop below the captain who reports to them.
    let (major, captain) = (role(t, None, true, "Major"), role(t, None, true, "Captain"));
    classify(major, Some(Rank::Major), None);
    classify(captain, Some(Rank::Captain), Some(major));
    let up = update_role(
        captain,
        "classification: { militaryOccupation: SIGNALS, rank: COLONEL }",
    )
    .await
    .unwrap_err();
    assert!(up.contains("not junior"), "{up}");
    let down = update_role(
        major,
        "classification: { militaryOccupation: SIGNALS, rank: LIEUTENANT }",
    )
    .await
    .unwrap_err();
    assert!(down.contains("Captain"), "{down}");
    assert!(
        update_role(
            major,
            "classification: { militaryOccupation: SIGNALS, rank: COLONEL }"
        )
        .await
        .is_ok()
    );
}
