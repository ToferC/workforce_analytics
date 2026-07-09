use async_graphql::*;
use chrono::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;
use diesel::prelude::*;

use crate::database::connection;
use crate::schema::{capabilities, validations, roles, requirements, works, teams, org_tiers};
use crate::models::{
    TimeBucket, TimeSeriesPoint, LabeledSeries,
    SupplyDemandPoint, SupplyDemandSeries,
    TeamCapabilityCell, TeamCapabilityRow, OrgTierCapabilityRow,
    TalentMovement,
    SkillDomain, CapabilityLevel, WorkStatus,
};
use crate::common_utils::{RoleGuard, UserRole};
use crate::graphql::loaders::off_executor;

/// GraphQL wire representation of a SkillDomain (SCREAMING_SNAKE_CASE), the
/// same conversion async-graphql applies automatically when a field's type is
/// `SkillDomain` itself. Needed anywhere a domain is folded into a plain
/// `String` field (map keys, series labels): `format!("{:?}", domain)` alone
/// yields Rust's PascalCase Debug output (e.g. "SoftwareEngineering"), which
/// doesn't match the "SOFTWARE_ENGINEERING" the frontend looks up against.
fn domain_graphql_key(domain: SkillDomain) -> String {
    let debug = format!("{:?}", domain);
    let mut key = String::with_capacity(debug.len() + 4);
    for (i, ch) in debug.char_indices() {
        if ch.is_ascii_uppercase() && i != 0 {
            key.push('_');
        }
        key.push(ch.to_ascii_uppercase());
    }
    key
}

/// Convert a CapabilityLevel to its numeric value (0-400)
fn level_value(level: CapabilityLevel) -> i64 {
    match level {
        CapabilityLevel::Desired => 0,
        CapabilityLevel::Novice => 100,
        CapabilityLevel::Experienced => 200,
        CapabilityLevel::Expert => 300,
        CapabilityLevel::Specialist => 400,
    }
}

/// Convert a CapabilityLevel to its weight as f64
pub(crate) fn level_weight(level: CapabilityLevel) -> f64 {
    level_value(level) as f64 / 100.0
}

/// Collect all descendant org_tier IDs (including the given id) using a pre-loaded list of (id, parent_tier)
pub(crate) fn collect_descendant_tier_ids(root: Uuid, all_tiers_raw: &[(Uuid, Option<Uuid>)]) -> Vec<Uuid> {
    let mut tier_ids: Vec<Uuid> = Vec::new();
    let mut queue = vec![root];
    while let Some(current) = queue.pop() {
        tier_ids.push(current);
        for (tid, parent) in all_tiers_raw {
            if *parent == Some(current) {
                queue.push(*tid);
            }
        }
    }
    tier_ids
}

/// Get all person_ids that have active roles under the given org_tier (and descendants)
pub(crate) fn get_person_ids_under_org_tier(org_tier_id: &Uuid) -> Result<Vec<Uuid>> {
    let mut conn = connection()?;

    let all_tiers_raw: Vec<(Uuid, Option<Uuid>)> = org_tiers::table
        .select((org_tiers::id, org_tiers::parent_tier))
        .load(&mut conn)?;

    let tier_ids = collect_descendant_tier_ids(*org_tier_id, &all_tiers_raw);

    let team_ids: Vec<Uuid> = teams::table
        .filter(teams::org_tier_id.eq_any(&tier_ids))
        .select(teams::id)
        .load::<Uuid>(&mut conn)?;

    let person_ids: Vec<Uuid> = roles::table
        .filter(roles::team_id.eq_any(&team_ids))
        .filter(roles::active.eq(true))
        .filter(roles::person_id.is_not_null())
        .select(roles::person_id)
        .load::<Option<Uuid>>(&mut conn)?
        .into_iter()
        .flatten()
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    Ok(person_ids)
}

/// Build supply-side data: capabilities and validations, returning (cap_rows, val_map)
fn load_supply_data(
    conn: &mut PgConnection,
    org_tier_id: Option<Uuid>,
) -> Result<(Vec<(Uuid, SkillDomain, Uuid, Option<NaiveDateTime>, CapabilityLevel, Option<CapabilityLevel>)>, HashMap<Uuid, Vec<(NaiveDateTime, i64)>>)> {
    type CapRow = (Uuid, SkillDomain, Uuid, Option<NaiveDateTime>, CapabilityLevel, Option<CapabilityLevel>);
    let mut cap_query = capabilities::table
        .select((
            capabilities::id,
            capabilities::domain,
            capabilities::person_id,
            capabilities::retired_at,
            capabilities::self_identified_level,
            capabilities::validated_level,
        ))
        .into_boxed();

    if let Some(tier_id) = org_tier_id {
        let person_ids = get_person_ids_under_org_tier(&tier_id)?;
        cap_query = cap_query.filter(capabilities::person_id.eq_any(person_ids));
    }

    let cap_rows: Vec<CapRow> = cap_query.load(conn)?;

    let val_rows: Vec<(Uuid, CapabilityLevel, NaiveDateTime)> = validations::table
        .select((
            validations::capability_id,
            validations::validated_level,
            validations::created_at,
        ))
        .load(conn)?;

    let mut val_map: HashMap<Uuid, Vec<(NaiveDateTime, i64)>> = HashMap::new();
    for (cap_id, level, created_at) in val_rows {
        val_map.entry(cap_id)
            .or_default()
            .push((created_at, level_value(level)));
    }

    Ok((cap_rows, val_map))
}

/// Compute supply value for a set of capabilities in a time bucket
fn compute_supply(
    domain_caps: &[&(Uuid, SkillDomain, Uuid, Option<NaiveDateTime>, CapabilityLevel, Option<CapabilityLevel>)],
    val_map: &HashMap<Uuid, Vec<(NaiveDateTime, i64)>>,
    period_start: NaiveDateTime,
    bend: NaiveDateTime,
) -> f64 {
    let mut supply = 0.0f64;
    for (cap_id, _, _, retired_at, _sil, _vl) in domain_caps {
        if let Some(r) = retired_at {
            if *r < period_start {
                continue;
            }
        }
        if let Some(vals) = val_map.get(cap_id) {
            let relevant: Vec<i64> = vals.iter()
                .filter(|(created_at, _)| *created_at <= bend)
                .map(|(_, v)| *v)
                .collect();
            if !relevant.is_empty() {
                let avg = relevant.iter().sum::<i64>() as f64 / relevant.len() as f64;
                supply += avg / 100.0;
            }
        }
    }
    supply
}

// ─── Capability Growth ───────────────────────────────────────────────

#[derive(Default)]
pub struct CapabilityGrowthQuery;

#[Object]
impl CapabilityGrowthQuery {
    /// Cumulative validated capability across the organization over time,
    /// one series per SkillDomain.
    #[graphql(guard = "RoleGuard::new(UserRole::User)")]
    pub async fn capability_growth(
        &self,
        bucket: TimeBucket,
        from: Option<NaiveDateTime>,
        to: Option<NaiveDateTime>,
        domain: Option<SkillDomain>,
        org_tier_id: Option<Uuid>,
    ) -> Result<Vec<LabeledSeries>> {
        // Table-scan-heavy; run off the async executor.
        off_executor(move || compute_capability_growth(bucket, from, to, domain, org_tier_id)).await
    }
}

fn compute_capability_growth(
    bucket: TimeBucket,
    from: Option<NaiveDateTime>,
    to: Option<NaiveDateTime>,
    domain: Option<SkillDomain>,
    org_tier_id: Option<Uuid>,
) -> Result<Vec<LabeledSeries>> {
    {
        let mut conn = connection()?;
        let (cap_rows, val_map) = load_supply_data(&mut conn, org_tier_id)?;

        let effective_from = from
            .or_else(|| val_map.values().flat_map(|v| v.iter().map(|(t, _)| *t)).min())
            .unwrap_or_else(|| Utc::now().naive_utc());
        let effective_to = to.unwrap_or_else(|| Utc::now().naive_utc());
        let bucket_starts = bucket.generate_buckets(effective_from, effective_to);

        let mut domains: Vec<SkillDomain> = cap_rows.iter().map(|(_, d, _, _, _, _)| *d).collect();
        domains.sort_by_key(|d| format!("{:?}", d));
        domains.dedup();
        if let Some(filter_domain) = domain {
            domains.retain(|d| *d == filter_domain);
        }

        let mut result: Vec<LabeledSeries> = Vec::new();

        for d in domains {
            let domain_caps: Vec<_> = cap_rows.iter().filter(|(_, cap_d, _, _, _, _)| *cap_d == d).collect();
            let points: Vec<TimeSeriesPoint> = bucket_starts.iter().map(|&period_start| {
                let bend = bucket.bucket_end(period_start);
                let value = compute_supply(&domain_caps, &val_map, period_start, bend);
                TimeSeriesPoint { period_start, bucket, value }
            }).collect();

            result.push(LabeledSeries { key: domain_graphql_key(d), points });
        }

        Ok(result)
    }
}

// ─── Supply/Demand ───────────────────────────────────────────────────

#[derive(Default)]
pub struct SupplyDemandQuery;

#[Object]
impl SupplyDemandQuery {
    /// Per-domain capability supply vs demand over time.
    #[graphql(guard = "RoleGuard::new(UserRole::User)")]
    pub async fn capability_supply_demand(
        &self,
        bucket: TimeBucket,
        from: Option<NaiveDateTime>,
        to: Option<NaiveDateTime>,
        domain: Option<SkillDomain>,
        org_tier_id: Option<Uuid>,
    ) -> Result<Vec<SupplyDemandSeries>> {
        // Table-scan-heavy; run off the async executor.
        off_executor(move || compute_capability_supply_demand(bucket, from, to, domain, org_tier_id)).await
    }
}

fn compute_capability_supply_demand(
    bucket: TimeBucket,
    from: Option<NaiveDateTime>,
    to: Option<NaiveDateTime>,
    domain: Option<SkillDomain>,
    org_tier_id: Option<Uuid>,
) -> Result<Vec<SupplyDemandSeries>> {
    {
        let mut conn = connection()?;
        let (cap_rows, val_map) = load_supply_data(&mut conn, org_tier_id)?;

        // ---- Demand: requirements + works ----
        type ReqRow = (Uuid, SkillDomain, CapabilityLevel, NaiveDateTime, Option<NaiveDateTime>);
        let mut req_query = requirements::table
            .select((
                requirements::id,
                requirements::domain,
                requirements::required_level,
                requirements::created_at,
                requirements::retired_at,
            ))
            .into_boxed();

        if let Some(tier_id) = org_tier_id {
            let all_tiers_raw: Vec<(Uuid, Option<Uuid>)> = org_tiers::table
                .select((org_tiers::id, org_tiers::parent_tier))
                .load(&mut conn)?;
            let tier_ids = collect_descendant_tier_ids(tier_id, &all_tiers_raw);
            let team_ids: Vec<Uuid> = teams::table
                .filter(teams::org_tier_id.eq_any(&tier_ids))
                .select(teams::id)
                .load::<Uuid>(&mut conn)?;
            let role_ids: Vec<Uuid> = roles::table
                .filter(roles::team_id.eq_any(&team_ids))
                .select(roles::id)
                .load::<Uuid>(&mut conn)?;
            req_query = req_query.filter(requirements::role_id.eq_any(role_ids));
        }

        let req_rows: Vec<ReqRow> = req_query.load(&mut conn)?;

        type WorkRow = (Uuid, SkillDomain, CapabilityLevel, NaiveDateTime);
        let mut work_query = works::table
            .select((
                works::id,
                works::domain,
                works::capability_level,
                works::created_at,
            ))
            .filter(works::work_status.ne_all(vec![WorkStatus::Completed, WorkStatus::Cancelled]))
            .into_boxed();

        if let Some(tier_id) = org_tier_id {
            let all_tiers_raw: Vec<(Uuid, Option<Uuid>)> = org_tiers::table
                .select((org_tiers::id, org_tiers::parent_tier))
                .load(&mut conn)?;
            let tier_ids = collect_descendant_tier_ids(tier_id, &all_tiers_raw);
            let team_ids: Vec<Uuid> = teams::table
                .filter(teams::org_tier_id.eq_any(&tier_ids))
                .select(teams::id)
                .load::<Uuid>(&mut conn)?;
            let role_ids: Vec<Uuid> = roles::table
                .filter(roles::team_id.eq_any(&team_ids))
                .select(roles::id)
                .load::<Uuid>(&mut conn)?;
            work_query = work_query.filter(works::role_id.eq_any(role_ids));
        }

        let work_rows: Vec<WorkRow> = work_query.load(&mut conn)?;

        // Time range
        let effective_from = from
            .or_else(|| val_map.values().flat_map(|v| v.iter().map(|(t, _)| *t)).min())
            .unwrap_or_else(|| Utc::now().naive_utc());
        let effective_to = to.unwrap_or_else(|| Utc::now().naive_utc());
        let bucket_starts = bucket.generate_buckets(effective_from, effective_to);

        let mut domains: Vec<SkillDomain> = cap_rows.iter().map(|(_, d, _, _, _, _)| *d)
            .chain(req_rows.iter().map(|(_, d, _, _, _)| *d))
            .chain(work_rows.iter().map(|(_, d, _, _)| *d))
            .collect();
        domains.sort_by_key(|d| format!("{:?}", d));
        domains.dedup();
        if let Some(filter_domain) = domain {
            domains.retain(|d| *d == filter_domain);
        }

        let mut result: Vec<SupplyDemandSeries> = Vec::new();

        for d in domains {
            let domain_caps: Vec<_> = cap_rows.iter().filter(|(_, cap_d, _, _, _, _)| *cap_d == d).collect();
            let domain_reqs: Vec<_> = req_rows.iter().filter(|(_, req_d, _, _, _)| *req_d == d).collect();
            let domain_works: Vec<_> = work_rows.iter().filter(|(_, wd, _, _)| *wd == d).collect();

            let mut points: Vec<SupplyDemandPoint> = Vec::new();

            for &period_start in &bucket_starts {
                let bend = bucket.bucket_end(period_start);
                let supply = compute_supply(&domain_caps, &val_map, period_start, bend);

                let mut demand = 0.0f64;
                for (_id, _dom, req_level, req_created, req_retired) in &domain_reqs {
                    if *req_created > bend { continue; }
                    if let Some(r) = req_retired {
                        if *r <= period_start { continue; }
                    }
                    demand += level_weight(*req_level);
                }
                for (_id, _dom, cap_level, work_created) in &domain_works {
                    if *work_created <= bend {
                        demand += level_weight(*cap_level);
                    }
                }

                points.push(SupplyDemandPoint { period_start, bucket, supply, demand });
            }

            result.push(SupplyDemandSeries { domain: domain_graphql_key(d), points });
        }

        Ok(result)
    }
}

// ─── Team Capability Matrix ──────────────────────────────────────────

#[derive(Default)]
pub struct TeamCapabilityMatrixQuery;

#[Object]
impl TeamCapabilityMatrixQuery {
    /// Capability depth per team across all skill domains (for a heatmap).
    #[graphql(guard = "RoleGuard::new(UserRole::User)")]
    pub async fn team_capability_matrix(
        &self,
        org_tier_id: Option<Uuid>,
    ) -> Result<Vec<TeamCapabilityRow>> {
        off_executor(move || compute_team_capability_matrix(org_tier_id, None)).await
    }

    /// The capability heatmap rolled up to a chosen org-tier level (default
    /// tier 2). Teams are grouped under their nearest ancestor tier at or
    /// above the requested level; a person contributing to several teams
    /// under the same tier counts once toward that tier's depth.
    #[graphql(guard = "RoleGuard::new(UserRole::User)")]
    pub async fn org_tier_capability_matrix(
        &self,
        #[graphql(default = 2)] tier_level: i32,
        org_tier_id: Option<Uuid>,
    ) -> Result<Vec<OrgTierCapabilityRow>> {
        off_executor(move || compute_org_tier_capability_matrix(tier_level, org_tier_id)).await
    }
}

/// Walk up the tier tree from `tier_id` to the nearest ancestor whose
/// tier_level is at or above (numerically ≤) `rollup_level`. A team attached
/// below the rollup level lands on its level-N ancestor; a team already at or
/// above it stays where it is. Cycles or dangling parents fall back to the
/// last tier seen.
pub(crate) fn rollup_ancestor(
    tier_id: Uuid,
    rollup_level: i32,
    tiers: &HashMap<Uuid, (Option<Uuid>, i32)>,
) -> Uuid {
    let mut current = tier_id;
    let mut seen = std::collections::HashSet::new();
    while let Some((parent, level)) = tiers.get(&current) {
        if *level <= rollup_level || !seen.insert(current) {
            return current;
        }
        match parent {
            Some(p) => current = *p,
            None => return current,
        }
    }
    current
}

/// The team capability matrix aggregated at a tier level: same two batched
/// queries as the per-team version, then grouped by rollup ancestor with
/// per-person dedup inside each tier.
pub(crate) fn compute_org_tier_capability_matrix(
    rollup_level: i32,
    org_tier_id: Option<Uuid>,
) -> Result<Vec<OrgTierCapabilityRow>> {
    let mut conn = connection()?;

    // Full tier tree: id -> (parent, level), plus display data.
    let tier_rows: Vec<(Uuid, Option<Uuid>, String, i32)> = org_tiers::table
        .select((org_tiers::id, org_tiers::parent_tier, org_tiers::name_en, org_tiers::tier_level))
        .load(&mut conn)?;
    let tier_tree: HashMap<Uuid, (Option<Uuid>, i32)> = tier_rows
        .iter()
        .map(|(id, parent, _, level)| (*id, (*parent, *level)))
        .collect();
    let tier_display: HashMap<Uuid, (String, i32)> = tier_rows
        .iter()
        .map(|(id, _, name, level)| (*id, (name.clone(), *level)))
        .collect();

    // Teams in scope (optionally restricted to a subtree, like the team matrix).
    let mut team_query = teams::table
        .select((teams::id, teams::org_tier_id))
        .into_boxed();
    if let Some(scope_tier) = org_tier_id {
        let all_tiers_raw: Vec<(Uuid, Option<Uuid>)> =
            tier_rows.iter().map(|(id, parent, _, _)| (*id, *parent)).collect();
        let tier_ids = collect_descendant_tier_ids(scope_tier, &all_tiers_raw);
        team_query = team_query.filter(teams::org_tier_id.eq_any(tier_ids));
    }
    let team_rows: Vec<(Uuid, Uuid)> = team_query.load(&mut conn)?;

    // Active memberships for those teams, in one query.
    let team_ids: Vec<Uuid> = team_rows.iter().map(|(id, _)| *id).collect();
    let membership_rows: Vec<(Uuid, Option<Uuid>)> = roles::table
        .filter(roles::team_id.eq_any(&team_ids))
        .filter(roles::active.eq(true))
        .filter(roles::person_id.is_not_null())
        .select((roles::team_id, roles::person_id))
        .load(&mut conn)?;

    // Group people under each team's rollup tier; a person on several teams
    // under the same tier counts once.
    let team_tier: HashMap<Uuid, Uuid> = team_rows
        .iter()
        .map(|(team_id, tier_id)| (*team_id, rollup_ancestor(*tier_id, rollup_level, &tier_tree)))
        .collect();

    let mut members_by_tier: HashMap<Uuid, std::collections::HashSet<Uuid>> = HashMap::new();
    let mut all_person_ids: std::collections::HashSet<Uuid> = std::collections::HashSet::new();
    for (team_id, person_id) in membership_rows {
        if let (Some(tier), Some(pid)) = (team_tier.get(&team_id), person_id) {
            members_by_tier.entry(*tier).or_default().insert(pid);
            all_person_ids.insert(pid);
        }
    }

    // All active capabilities for the people involved, in one query.
    type CapRow = (SkillDomain, Uuid, Option<CapabilityLevel>, CapabilityLevel);
    let cap_rows: Vec<CapRow> = if all_person_ids.is_empty() {
        Vec::new()
    } else {
        capabilities::table
            .filter(capabilities::person_id.eq_any(&all_person_ids))
            .filter(capabilities::retired_at.is_null())
            .select((
                capabilities::domain,
                capabilities::person_id,
                capabilities::validated_level,
                capabilities::self_identified_level,
            ))
            .load::<CapRow>(&mut conn)?
    };

    let mut caps_by_person: HashMap<Uuid, Vec<(SkillDomain, f64)>> = HashMap::new();
    for (domain, person_id, validated_level, self_identified_level) in cap_rows {
        let weight = validated_level
            .map(level_weight)
            .unwrap_or_else(|| level_weight(self_identified_level));
        caps_by_person.entry(person_id).or_default().push((domain, weight));
    }

    let mut result: Vec<OrgTierCapabilityRow> = Vec::new();
    for (tier_id, person_ids) in members_by_tier {
        let (org_tier_name, tier_level) = tier_display
            .get(&tier_id)
            .cloned()
            .unwrap_or_else(|| ("Unknown tier".to_string(), rollup_level));

        let mut domain_map: HashMap<String, (f64, std::collections::HashSet<Uuid>)> = HashMap::new();
        for person_id in &person_ids {
            if let Some(caps) = caps_by_person.get(person_id) {
                for (domain, weight) in caps {
                    let entry = domain_map.entry(domain_graphql_key(*domain)).or_default();
                    entry.0 += weight;
                    entry.1.insert(*person_id);
                }
            }
        }

        let cells: Vec<TeamCapabilityCell> = domain_map
            .into_iter()
            .filter(|(_, (depth, _))| *depth > 0.0)
            .map(|(domain, (depth, people))| TeamCapabilityCell {
                domain,
                depth,
                people_count: people.len() as i32,
            })
            .collect();

        result.push(OrgTierCapabilityRow { org_tier_id: tier_id, org_tier_name, tier_level, cells });
    }

    // Stable, readable order for the heatmap rows.
    result.sort_by(|a, b| a.org_tier_name.to_lowercase().cmp(&b.org_tier_name.to_lowercase()));

    Ok(result)
}

/// Shared implementation for team capability matrix computation.
/// If `single_team_id` is Some, only computes for that team.
pub(crate) fn compute_team_capability_matrix(
    org_tier_id: Option<Uuid>,
    single_team_id: Option<Uuid>,
) -> Result<Vec<TeamCapabilityRow>> {
    let mut conn = connection()?;

    let team_rows: Vec<(Uuid, String)> = if let Some(team_id) = single_team_id {
        teams::table
            .filter(teams::id.eq(team_id))
            .select((teams::id, teams::name_en))
            .load::<(Uuid, String)>(&mut conn)?
    } else if let Some(tier_id) = org_tier_id {
        let all_tiers_raw: Vec<(Uuid, Option<Uuid>)> = org_tiers::table
            .select((org_tiers::id, org_tiers::parent_tier))
            .load(&mut conn)?;
        let tier_ids = collect_descendant_tier_ids(tier_id, &all_tiers_raw);
        teams::table
            .filter(teams::org_tier_id.eq_any(&tier_ids))
            .select((teams::id, teams::name_en))
            .load::<(Uuid, String)>(&mut conn)?
    } else {
        teams::table
            .select((teams::id, teams::name_en))
            .load::<(Uuid, String)>(&mut conn)?
    };

    // Two batched queries for the whole matrix (was two queries *per team*):
    // all active team memberships, then all active capabilities for the
    // people involved. Everything else is grouped in memory.
    let team_ids: Vec<Uuid> = team_rows.iter().map(|(id, _)| *id).collect();

    let membership_rows: Vec<(Uuid, Option<Uuid>)> = roles::table
        .filter(roles::team_id.eq_any(&team_ids))
        .filter(roles::active.eq(true))
        .filter(roles::person_id.is_not_null())
        .select((roles::team_id, roles::person_id))
        .load(&mut conn)?;

    // Sets, not Vecs: a person holding two active roles on the same team
    // still counts once toward that team's capability depth.
    let mut members_by_team: HashMap<Uuid, std::collections::HashSet<Uuid>> = HashMap::new();
    let mut all_person_ids: std::collections::HashSet<Uuid> = std::collections::HashSet::new();
    for (team_id, person_id) in membership_rows.into_iter() {
        if let Some(pid) = person_id {
            members_by_team.entry(team_id).or_default().insert(pid);
            all_person_ids.insert(pid);
        }
    }

    type CapRow2 = (SkillDomain, Uuid, Option<CapabilityLevel>, CapabilityLevel);
    let cap_rows: Vec<CapRow2> = if all_person_ids.is_empty() {
        Vec::new()
    } else {
        capabilities::table
            .filter(capabilities::person_id.eq_any(&all_person_ids))
            .filter(capabilities::retired_at.is_null())
            .select((
                capabilities::domain,
                capabilities::person_id,
                capabilities::validated_level,
                capabilities::self_identified_level,
            ))
            .load::<CapRow2>(&mut conn)?
    };

    let mut caps_by_person: HashMap<Uuid, Vec<(SkillDomain, f64)>> = HashMap::new();
    for (domain, person_id, validated_level, self_identified_level) in cap_rows {
        let weight = if let Some(vl) = validated_level {
            level_weight(vl)
        } else {
            level_weight(self_identified_level)
        };
        caps_by_person.entry(person_id).or_default().push((domain, weight));
    }

    let mut result: Vec<TeamCapabilityRow> = Vec::new();

    for (team_id, team_name) in team_rows {
        let mut domain_map: HashMap<String, (f64, std::collections::HashSet<Uuid>)> = HashMap::new();
        if let Some(person_ids) = members_by_team.get(&team_id) {
            for person_id in person_ids {
                if let Some(caps) = caps_by_person.get(person_id) {
                    for (domain, weight) in caps {
                        let entry = domain_map.entry(domain_graphql_key(*domain)).or_default();
                        entry.0 += weight;
                        entry.1.insert(*person_id);
                    }
                }
            }
        }

        let cells: Vec<TeamCapabilityCell> = domain_map.into_iter()
            .filter(|(_, (depth, _))| *depth > 0.0)
            .map(|(domain, (depth, people))| TeamCapabilityCell {
                domain,
                depth,
                people_count: people.len() as i32,
            })
            .collect();

        result.push(TeamCapabilityRow { team_id, team_name, cells });
    }

    Ok(result)
}

// ─── Talent Movements ────────────────────────────────────────────────

#[derive(Default)]
pub struct TalentMovementQuery;

#[Object]
impl TalentMovementQuery {
    /// Derived role transitions over a window, for mobility/promotion analysis.
    #[graphql(guard = "RoleGuard::new(UserRole::User)")]
    pub async fn talent_movements(
        &self,
        from: Option<NaiveDateTime>,
        to: Option<NaiveDateTime>,
        org_tier_id: Option<Uuid>,
    ) -> Result<Vec<TalentMovement>> {
        // Table-scan-heavy; run off the async executor.
        off_executor(move || compute_talent_movements(from, to, org_tier_id)).await
    }
}

fn compute_talent_movements(
    from: Option<NaiveDateTime>,
    to: Option<NaiveDateTime>,
    org_tier_id: Option<Uuid>,
) -> Result<Vec<TalentMovement>> {
    {
        let mut conn = connection()?;

        type RoleRowNullable = (Uuid, Option<Uuid>, Uuid, NaiveDateTime, Option<crate::models::Rank>, Option<i32>);
        let role_rows_raw: Vec<RoleRowNullable> = roles::table
            .filter(roles::person_id.is_not_null())
            .select((
                roles::id,
                roles::person_id,
                roles::team_id,
                roles::start_datestamp,
                roles::rank,
                roles::occupational_level,
            ))
            .order_by(roles::start_datestamp)
            .load::<RoleRowNullable>(&mut conn)?;

        type RoleRow = (Uuid, Uuid, Uuid, NaiveDateTime, Option<crate::models::Rank>, Option<i32>);
        let role_rows: Vec<RoleRow> = role_rows_raw.into_iter()
            .filter_map(|(id, pid, tid, start, rank, occ)| {
                pid.map(|p| (id, p, tid, start, rank, occ))
            })
            .collect();

        let mut person_roles: HashMap<Uuid, Vec<(Uuid, Uuid, NaiveDateTime, Option<crate::models::Rank>, Option<i32>)>> = HashMap::new();
        for (role_id, person_id, team_id, start, rank, occ_level) in role_rows {
            person_roles.entry(person_id)
                .or_default()
                .push((role_id, team_id, start, rank, occ_level));
        }

        let filtered_team_ids: Option<std::collections::HashSet<Uuid>> = if let Some(tier_id) = org_tier_id {
            let all_tiers_raw: Vec<(Uuid, Option<Uuid>)> = org_tiers::table
                .select((org_tiers::id, org_tiers::parent_tier))
                .load(&mut conn)?;
            let tier_ids = collect_descendant_tier_ids(tier_id, &all_tiers_raw);
            let tids: Vec<Uuid> = teams::table
                .filter(teams::org_tier_id.eq_any(&tier_ids))
                .select(teams::id)
                .load::<Uuid>(&mut conn)?;
            Some(tids.into_iter().collect())
        } else {
            None
        };

        let rank_order = |r: &crate::models::Rank| -> i32 {
            use crate::models::Rank::*;
            match r {
                Private => 1,
                Corporal => 2,
                MasterCorporal => 3,
                Sergeant => 4,
                WarrantOfficer => 5,
                MasterWarrantOfficer => 6,
                ChiefWarrantOfficer => 7,
                SecondLieutenant => 8,
                Lieutenant => 9,
                Captain => 10,
                Major => 11,
                LieutenantColonel => 12,
                Colonel => 13,
                BrigadierGeneral => 14,
                MajorGeneral => 15,
                LieutenantGeneral => 16,
                General => 17,
            }
        };

        let level_str = |rank: Option<crate::models::Rank>, occ_level: Option<i32>| -> Option<String> {
            if let Some(r) = rank {
                Some(format!("{:?}", r))
            } else {
                occ_level.map(|l| l.to_string())
            }
        };

        let mut movements: Vec<TalentMovement> = Vec::new();

        for (person_id, mut person_role_list) in person_roles {
            person_role_list.sort_by_key(|(_, _, start, _, _)| *start);

            if let Some((_role_id, team_id, start, rank, occ_level)) = person_role_list.first() {
                let movement = TalentMovement {
                    person_id,
                    at: *start,
                    from_team_id: None,
                    to_team_id: Some(*team_id),
                    from_level: None,
                    to_level: level_str(*rank, *occ_level),
                    kind: "INFLOW".to_string(),
                };

                let in_scope = filtered_team_ids.as_ref()
                    .map(|fts| fts.contains(team_id))
                    .unwrap_or(true);
                let in_time = from.map(|f| *start >= f).unwrap_or(true)
                    && to.map(|t| *start <= t).unwrap_or(true);

                if in_scope && in_time {
                    movements.push(movement);
                }
            }

            for i in 1..person_role_list.len() {
                let (_prev_id, prev_team, _prev_start, prev_rank, prev_occ) = &person_role_list[i - 1];
                let (_curr_id, curr_team, curr_start, curr_rank, curr_occ) = &person_role_list[i];

                let kind = if let (Some(pr), Some(cr)) = (prev_rank, curr_rank) {
                    if rank_order(cr) > rank_order(pr) { "PROMOTION" } else { "LATERAL" }
                } else {
                    "LATERAL"
                };

                let in_scope = filtered_team_ids.as_ref()
                    .map(|fts| fts.contains(prev_team) || fts.contains(curr_team))
                    .unwrap_or(true);
                let in_time = from.map(|f| *curr_start >= f).unwrap_or(true)
                    && to.map(|t| *curr_start <= t).unwrap_or(true);

                if in_scope && in_time {
                    movements.push(TalentMovement {
                        person_id,
                        at: *curr_start,
                        from_team_id: Some(*prev_team),
                        to_team_id: Some(*curr_team),
                        from_level: level_str(*prev_rank, *prev_occ),
                        to_level: level_str(*curr_rank, *curr_occ),
                        kind: kind.to_string(),
                    });
                }
            }
        }

        movements.sort_by_key(|m| m.at);
        Ok(movements)
    }
}

// ─── Combined (kept for backward compat as a merged object) ─────────

#[derive(Default, MergedObject)]
pub struct AnalyticsQuery(
    CapabilityGrowthQuery,
    SupplyDemandQuery,
    TeamCapabilityMatrixQuery,
    TalentMovementQuery,
);

#[cfg(test)]
mod tests {
    use super::*;

    fn dt(s: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").unwrap()
    }

    #[test]
    fn domain_graphql_key_matches_wire_format() {
        assert_eq!(domain_graphql_key(SkillDomain::CyberSecurity), "CYBER_SECURITY");
        assert_eq!(domain_graphql_key(SkillDomain::Strategy), "STRATEGY");
        // The regression that motivated this helper: multi-word domains must
        // not come out as PascalCase Debug output.
        assert_ne!(domain_graphql_key(SkillDomain::SoftwareEngineering), "SoftwareEngineering");
        assert_eq!(domain_graphql_key(SkillDomain::SoftwareEngineering), "SOFTWARE_ENGINEERING");
    }

    #[test]
    fn level_values_step_by_hundred() {
        assert_eq!(level_value(CapabilityLevel::Desired), 0);
        assert_eq!(level_value(CapabilityLevel::Specialist), 400);
        assert_eq!(level_weight(CapabilityLevel::Experienced), 2.0);
    }

    #[test]
    fn descendant_tiers_include_root_and_transitive_children() {
        let (root, child, grandchild, unrelated) =
            (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
        let tiers = vec![
            (root, None),
            (child, Some(root)),
            (grandchild, Some(child)),
            (unrelated, None),
        ];

        let ids = collect_descendant_tier_ids(root, &tiers);
        assert!(ids.contains(&root));
        assert!(ids.contains(&child));
        assert!(ids.contains(&grandchild));
        assert!(!ids.contains(&unrelated));
        assert_eq!(ids.len(), 3);
    }

    #[test]
    fn supply_averages_validations_up_to_bucket_end() {
        let cap_id = Uuid::new_v4();
        let row = (cap_id, SkillDomain::CyberSecurity, Uuid::new_v4(), None, CapabilityLevel::Expert, Some(CapabilityLevel::Expert));
        let caps = vec![&row];

        let mut val_map: HashMap<Uuid, Vec<(NaiveDateTime, i64)>> = HashMap::new();
        val_map.insert(cap_id, vec![
            (dt("2025-01-15 00:00:00"), 200), // Experienced
            (dt("2025-06-15 00:00:00"), 400), // Specialist
        ]);

        // Bucket ending before the second validation only sees the first.
        let supply_early = compute_supply(&caps, &val_map, dt("2025-01-01 00:00:00"), dt("2025-03-31 00:00:00"));
        assert!((supply_early - 2.0).abs() < 1e-9);

        // Bucket ending after both averages them: (200 + 400) / 2 / 100 = 3.
        let supply_late = compute_supply(&caps, &val_map, dt("2025-01-01 00:00:00"), dt("2025-12-31 00:00:00"));
        assert!((supply_late - 3.0).abs() < 1e-9);
    }

    #[test]
    fn supply_skips_capabilities_retired_before_the_bucket() {
        let cap_id = Uuid::new_v4();
        let retired = Some(dt("2024-12-31 00:00:00"));
        let row = (cap_id, SkillDomain::CyberSecurity, Uuid::new_v4(), retired, CapabilityLevel::Expert, None);
        let caps = vec![&row];

        let mut val_map: HashMap<Uuid, Vec<(NaiveDateTime, i64)>> = HashMap::new();
        val_map.insert(cap_id, vec![(dt("2024-06-01 00:00:00"), 300)]);

        let supply = compute_supply(&caps, &val_map, dt("2025-01-01 00:00:00"), dt("2025-03-31 00:00:00"));
        assert_eq!(supply, 0.0);
    }
}

#[cfg(test)]
mod rollup_tests {
    use super::*;

    #[test]
    fn rollup_walks_up_to_the_requested_level() {
        let (t1, t2, t3, t4) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
        let mut tiers: HashMap<Uuid, (Option<Uuid>, i32)> = HashMap::new();
        tiers.insert(t1, (None, 1));
        tiers.insert(t2, (Some(t1), 2));
        tiers.insert(t3, (Some(t2), 3));
        tiers.insert(t4, (Some(t3), 4));

        // A deep tier rolls up to its level-2 ancestor.
        assert_eq!(rollup_ancestor(t4, 2, &tiers), t2);
        assert_eq!(rollup_ancestor(t3, 2, &tiers), t2);
        // A tier already at the level stays put.
        assert_eq!(rollup_ancestor(t2, 2, &tiers), t2);
        // A tier above the level stays where it is.
        assert_eq!(rollup_ancestor(t1, 2, &tiers), t1);
    }

    #[test]
    fn rollup_handles_unknown_and_cyclic_tiers() {
        let orphan = Uuid::new_v4();
        let tiers: HashMap<Uuid, (Option<Uuid>, i32)> = HashMap::new();
        // Unknown tier id: fall back to itself rather than panicking.
        assert_eq!(rollup_ancestor(orphan, 2, &tiers), orphan);

        // A cycle below the rollup level terminates at the revisited node.
        let (a, b) = (Uuid::new_v4(), Uuid::new_v4());
        let mut cyclic: HashMap<Uuid, (Option<Uuid>, i32)> = HashMap::new();
        cyclic.insert(a, (Some(b), 4));
        cyclic.insert(b, (Some(a), 4));
        let landed = rollup_ancestor(a, 2, &cyclic);
        assert!(landed == a || landed == b);
    }
}
