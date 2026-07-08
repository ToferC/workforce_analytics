//! Shared span-of-control / ownership authorization helpers.
//!
//! A "manager" is a [`Person`](crate::models::Person) who owns an
//! [`OrgTier`](crate::models::OrgTier) (via `org_tier_ownerships`) or a
//! [`Team`](crate::models::Team) (via `team_ownerships`). Their **span of
//! control** is their directly owned tiers plus all descendant tiers, and the
//! teams / roles / people within them.
//!
//! Phase 0 of the Person-as-User / manager-scoped authorization roadmap lifts
//! the tier-tree traversal out of `analytics_query.rs` so both models and
//! GraphQL guards can reuse it. The pure traversal functions take a pre-loaded
//! adjacency list so they can be unit-tested without a database; the DB-backed
//! helpers compute a manager's span on demand (per-request caching arrives in
//! Phase 2).

use std::collections::HashSet;

use async_graphql::Result;
use diesel::prelude::*;
use uuid::Uuid;

use crate::database::connection;
use crate::schema::{org_tier_ownerships, org_tiers, roles, team_ownerships, teams};

/// Collect all descendant org_tier IDs (including `root` itself) from a
/// pre-loaded `(id, parent_tier)` adjacency list.
///
/// Pure and side-effect free so it can be unit-tested without a database.
pub fn collect_descendant_tier_ids(root: Uuid, all_tiers_raw: &[(Uuid, Option<Uuid>)]) -> Vec<Uuid> {
    collect_descendant_tier_ids_many(&[root], all_tiers_raw)
}

/// Like [`collect_descendant_tier_ids`] but seeded from multiple roots. The
/// result is de-duplicated, so overlapping subtrees (e.g. a manager who owns
/// both an ancestor tier and one of its descendants) are visited only once.
/// The `seen` set also guards against accidental cycles in the tier data.
pub fn collect_descendant_tier_ids_many(
    roots: &[Uuid],
    all_tiers_raw: &[(Uuid, Option<Uuid>)],
) -> Vec<Uuid> {
    let mut seen: HashSet<Uuid> = HashSet::new();
    let mut queue: Vec<Uuid> = roots.to_vec();

    while let Some(current) = queue.pop() {
        if !seen.insert(current) {
            continue;
        }
        for (tid, parent) in all_tiers_raw {
            if *parent == Some(current) {
                queue.push(*tid);
            }
        }
    }

    seen.into_iter().collect()
}

/// Compute the set of org_tier IDs a person manages: tiers owned directly (via
/// `org_tier_ownerships`) plus tiers owned through a team (`team_ownerships` →
/// team → `org_tier_id`), expanded to include every descendant tier.
///
/// Returns an empty set for a person who owns nothing (i.e. is not a manager).
pub fn managed_tier_ids_for_person(person_id: &Uuid) -> Result<HashSet<Uuid>> {
    let mut conn = connection()?;

    // Tiers owned directly.
    let mut roots: Vec<Uuid> = org_tier_ownerships::table
        .filter(org_tier_ownerships::owner_id.eq(person_id))
        .select(org_tier_ownerships::org_tier_id)
        .load::<Uuid>(&mut conn)?;

    // Tiers owned via team ownership (team -> org_tier).
    let owned_team_ids: Vec<Uuid> = team_ownerships::table
        .filter(team_ownerships::person_id.eq(person_id))
        .select(team_ownerships::team_id)
        .load::<Uuid>(&mut conn)?;

    if !owned_team_ids.is_empty() {
        let mut team_tier_ids: Vec<Uuid> = teams::table
            .filter(teams::id.eq_any(&owned_team_ids))
            .select(teams::org_tier_id)
            .load::<Uuid>(&mut conn)?;
        roots.append(&mut team_tier_ids);
    }

    if roots.is_empty() {
        return Ok(HashSet::new());
    }

    let all_tiers_raw: Vec<(Uuid, Option<Uuid>)> = org_tiers::table
        .select((org_tiers::id, org_tiers::parent_tier))
        .load(&mut conn)?;

    Ok(collect_descendant_tier_ids_many(&roots, &all_tiers_raw)
        .into_iter()
        .collect())
}

/// True if `person_id` manages `tier_id` (it is within their span of control).
pub fn can_manage_org_tier(person_id: &Uuid, tier_id: &Uuid) -> Result<bool> {
    let managed = managed_tier_ids_for_person(person_id)?;
    Ok(managed.contains(tier_id))
}

/// True if `person_id` manages the tier that `team_id` belongs to.
pub fn can_manage_team(person_id: &Uuid, team_id: &Uuid) -> Result<bool> {
    let managed = managed_tier_ids_for_person(person_id)?;
    if managed.is_empty() {
        return Ok(false);
    }

    let mut conn = connection()?;
    let tier_id: Uuid = teams::table
        .filter(teams::id.eq(team_id))
        .select(teams::org_tier_id)
        .first(&mut conn)?;

    Ok(managed.contains(&tier_id))
}

/// True if `person_id` manages the tier of the team that `role_id` belongs to.
pub fn can_manage_role(person_id: &Uuid, role_id: &Uuid) -> Result<bool> {
    let managed = managed_tier_ids_for_person(person_id)?;
    if managed.is_empty() {
        return Ok(false);
    }

    let mut conn = connection()?;
    let team_id: Uuid = roles::table
        .filter(roles::id.eq(role_id))
        .select(roles::team_id)
        .first(&mut conn)?;

    let tier_id: Uuid = teams::table
        .filter(teams::id.eq(team_id))
        .select(teams::org_tier_id)
        .first(&mut conn)?;

    Ok(managed.contains(&tier_id))
}

/// True if `target_person_id` holds an active role on any team whose org_tier is
/// within `person_id`'s span of control.
pub fn can_manage_person(person_id: &Uuid, target_person_id: &Uuid) -> Result<bool> {
    let managed = managed_tier_ids_for_person(person_id)?;
    if managed.is_empty() {
        return Ok(false);
    }

    let mut conn = connection()?;
    let team_ids: Vec<Uuid> = roles::table
        .filter(roles::person_id.eq(target_person_id))
        .filter(roles::active.eq(true))
        .select(roles::team_id)
        .load::<Uuid>(&mut conn)?;

    if team_ids.is_empty() {
        return Ok(false);
    }

    let tier_ids: Vec<Uuid> = teams::table
        .filter(teams::id.eq_any(&team_ids))
        .select(teams::org_tier_id)
        .load::<Uuid>(&mut conn)?;

    Ok(tier_ids.iter().any(|t| managed.contains(t)))
}

#[cfg(test)]
mod tests {
    use super::*;

    // A small multi-level tier tree:
    //
    //   root
    //   ├── a
    //   │   ├── a1
    //   │   └── a2
    //   └── b
    //       └── b1
    //   other (unrelated root)
    fn fixture() -> (
        Uuid, // root
        Uuid, // a
        Uuid, // a1
        Uuid, // a2
        Uuid, // b
        Uuid, // b1
        Uuid, // other
        Vec<(Uuid, Option<Uuid>)>,
    ) {
        let root = Uuid::new_v4();
        let a = Uuid::new_v4();
        let a1 = Uuid::new_v4();
        let a2 = Uuid::new_v4();
        let b = Uuid::new_v4();
        let b1 = Uuid::new_v4();
        let other = Uuid::new_v4();

        let tiers = vec![
            (root, None),
            (a, Some(root)),
            (a1, Some(a)),
            (a2, Some(a)),
            (b, Some(root)),
            (b1, Some(b)),
            (other, None),
        ];

        (root, a, a1, a2, b, b1, other, tiers)
    }

    #[test]
    fn direct_owner_sees_whole_subtree() {
        let (root, a, a1, a2, b, b1, other, tiers) = fixture();
        let span: HashSet<Uuid> = collect_descendant_tier_ids(root, &tiers).into_iter().collect();

        // Root manager spans every tier below root.
        for t in [root, a, a1, a2, b, b1] {
            assert!(span.contains(&t), "root span should include {t}");
        }
        // The unrelated tier is out of span.
        assert!(!span.contains(&other), "root span must not include unrelated tier");
    }

    #[test]
    fn mid_level_owner_sees_only_its_branch() {
        let (root, a, a1, a2, _b, b1, _other, tiers) = fixture();
        let span: HashSet<Uuid> = collect_descendant_tier_ids(a, &tiers).into_iter().collect();

        assert!(span.contains(&a));
        assert!(span.contains(&a1));
        assert!(span.contains(&a2));
        // Does not climb up to the ancestor, nor cross into sibling branch b.
        assert!(!span.contains(&root), "must not include ancestor");
        assert!(!span.contains(&b1), "must not include sibling branch");
    }

    #[test]
    fn leaf_owner_sees_only_itself() {
        let (_root, _a, a1, ..) = fixture();
        let tiers = fixture().7;
        let span = collect_descendant_tier_ids(a1, &tiers);
        assert_eq!(span, vec![a1]);
    }

    #[test]
    fn multiple_overlapping_roots_dedupe() {
        let (root, a, a1, a2, b, b1, other, tiers) = fixture();
        // Owns both an ancestor (root) and a descendant (a1) -> a1 must appear once.
        let span = collect_descendant_tier_ids_many(&[root, a1], &tiers);
        let unique: HashSet<Uuid> = span.iter().copied().collect();
        assert_eq!(span.len(), unique.len(), "no duplicate tier ids");
        for t in [root, a, a1, a2, b, b1] {
            assert!(unique.contains(&t));
        }
        assert!(!unique.contains(&other));
    }

    #[test]
    fn cyclic_data_terminates() {
        // Defensive: a cycle in parent pointers must not hang.
        let x = Uuid::new_v4();
        let y = Uuid::new_v4();
        let tiers = vec![(x, Some(y)), (y, Some(x))];
        let span: HashSet<Uuid> = collect_descendant_tier_ids(x, &tiers).into_iter().collect();
        assert!(span.contains(&x));
        assert!(span.contains(&y));
    }
}
