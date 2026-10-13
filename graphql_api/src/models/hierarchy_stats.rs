//! Headcount, capacity and vacancy roll-ups shared by `Team`, `OrgTier` (its
//! whole subtree) and `Organization` (all of its teams).
//!
//! Every scope reduces to a set of team ids, and `for_scopes` answers any
//! number of scopes with two grouped queries. The `*StatsLoader`s batch the
//! scopes a list asks for, so 25 index rows cost the same few queries as one.

use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use async_graphql::Result;
use diesel::dsl::sum;
use diesel::prelude::*;
use uuid::Uuid;

use crate::database::connection;
use crate::graphql::query::collect_descendant_tier_ids;
use crate::models::WorkStatus;
use crate::schema::{org_tiers, roles, teams, works};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HierarchyStats {
    /// Distinct people holding active roles.
    pub headcount: i32,
    /// Effort of work that is neither completed nor cancelled, on active roles.
    pub total_effort: i32,
    /// Active roles with no incumbent.
    pub vacant_role_count: i32,
}

impl HierarchyStats {
    /// Stats for each team on its own.
    pub fn for_teams(team_ids: &[Uuid]) -> Result<HashMap<Uuid, Self>> {
        Self::for_scopes(&team_ids.iter().map(|&id| (id, vec![id])).collect())
    }

    /// Stats for each tier over its whole subtree (the tier and every
    /// descendant tier), reusing the subtree walk behind the tier analytics.
    pub fn for_tiers(tier_ids: &[Uuid]) -> Result<HashMap<Uuid, Self>> {
        let mut conn = connection()?;
        let all_tiers: Vec<(Uuid, Option<Uuid>)> = org_tiers::table
            .select((org_tiers::id, org_tiers::parent_tier))
            .load(&mut conn)?;
        let subtrees: HashMap<Uuid, Vec<Uuid>> = tier_ids
            .iter()
            .map(|&id| (id, collect_descendant_tier_ids(id, &all_tiers)))
            .collect();

        let in_scope: HashSet<Uuid> = subtrees.values().flatten().copied().collect();
        let mut teams_by_tier: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        for (team_id, tier_id) in teams::table
            .filter(teams::org_tier_id.eq_any(&in_scope))
            .select((teams::id, teams::org_tier_id))
            .load::<(Uuid, Uuid)>(&mut conn)?
        {
            teams_by_tier.entry(tier_id).or_default().push(team_id);
        }

        let scopes = subtrees
            .into_iter()
            .map(|(root, tiers)| {
                let team_ids = tiers
                    .iter()
                    .filter_map(|t| teams_by_tier.get(t))
                    .flatten()
                    .copied()
                    .collect();
                (root, team_ids)
            })
            .collect();
        Self::for_scopes(&scopes)
    }

    /// Stats for each organization over all of its teams, i.e. across every
    /// tier it owns.
    pub fn for_organizations(organization_ids: &[Uuid]) -> Result<HashMap<Uuid, Self>> {
        let mut conn = connection()?;
        let mut scopes: HashMap<Uuid, Vec<Uuid>> = organization_ids
            .iter()
            .map(|&id| (id, Vec::new()))
            .collect();
        for (organization_id, team_id) in teams::table
            .filter(teams::organization_id.eq_any(organization_ids))
            .select((teams::organization_id, teams::id))
            .load::<(Uuid, Uuid)>(&mut conn)?
        {
            scopes.entry(organization_id).or_default().push(team_id);
        }
        Self::for_scopes(&scopes)
    }

    /// Stats per scope, where each scope is a set of team ids.
    fn for_scopes<K: Eq + Hash + Copy>(scopes: &HashMap<K, Vec<Uuid>>) -> Result<HashMap<K, Self>> {
        let team_ids: HashSet<Uuid> = scopes.values().flatten().copied().collect();
        let mut conn = connection()?;

        let incumbents: Vec<(Uuid, Option<Uuid>)> = roles::table
            .filter(roles::team_id.eq_any(&team_ids))
            .filter(roles::active.eq(true))
            .select((roles::team_id, roles::person_id))
            .load(&mut conn)?;

        let effort: HashMap<Uuid, i64> = works::table
            .inner_join(roles::table)
            .filter(roles::team_id.eq_any(&team_ids))
            .filter(roles::active.eq(true))
            .filter(works::work_status.ne_all(vec![WorkStatus::Cancelled, WorkStatus::Completed]))
            .group_by(roles::team_id)
            .select((roles::team_id, sum(works::effort)))
            .load::<(Uuid, Option<i64>)>(&mut conn)?
            .into_iter()
            .map(|(team_id, total)| (team_id, total.unwrap_or(0)))
            .collect();

        Ok(aggregate(scopes, &incumbents, &effort))
    }
}

/// Pure roll-up: `incumbents` holds one `(team_id, person_id)` row per active
/// role (`None` = vacant) and `effort` the active effort per team.
fn aggregate<K: Eq + Hash + Copy>(
    scopes: &HashMap<K, Vec<Uuid>>,
    incumbents: &[(Uuid, Option<Uuid>)],
    effort: &HashMap<Uuid, i64>,
) -> HashMap<K, HierarchyStats> {
    let mut by_team: HashMap<Uuid, Vec<Option<Uuid>>> = HashMap::new();
    for &(team_id, person_id) in incumbents {
        by_team.entry(team_id).or_default().push(person_id);
    }

    scopes
        .iter()
        .map(|(&key, team_ids)| {
            let holders: Vec<Option<Uuid>> = team_ids
                .iter()
                .filter_map(|t| by_team.get(t))
                .flatten()
                .copied()
                .collect();
            let people: HashSet<Uuid> = holders.iter().flatten().copied().collect();
            let stats = HierarchyStats {
                headcount: people.len() as i32,
                total_effort: team_ids.iter().filter_map(|t| effort.get(t)).sum::<i64>() as i32,
                vacant_role_count: holders.iter().filter(|p| p.is_none()).count() as i32,
            };
            (key, stats)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregate_counts_distinct_people_vacancies_and_effort_per_scope() {
        let (team_a, team_b, alice, bob) = (
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        );
        let incumbents = vec![
            (team_a, Some(alice)),
            (team_a, None),
            (team_b, Some(alice)), // Alice counts once across both teams.
            (team_b, Some(bob)),
            (team_b, None),
        ];
        let effort = HashMap::from([(team_a, 3), (team_b, 4)]);
        let scopes = HashMap::from([
            ("a", vec![team_a]),
            ("both", vec![team_a, team_b]),
            ("empty", vec![]),
        ]);

        let stats = aggregate(&scopes, &incumbents, &effort);

        assert_eq!(
            stats["a"],
            HierarchyStats {
                headcount: 1,
                total_effort: 3,
                vacant_role_count: 1
            }
        );
        assert_eq!(
            stats["both"],
            HierarchyStats {
                headcount: 2,
                total_effort: 7,
                vacant_role_count: 2
            }
        );
        assert_eq!(stats["empty"], HierarchyStats::default());
    }
}
