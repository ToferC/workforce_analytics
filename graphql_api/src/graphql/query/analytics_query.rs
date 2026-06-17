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
    TeamCapabilityCell, TeamCapabilityRow,
    TalentMovement,
    SkillDomain, CapabilityLevel, WorkStatus,
};
use crate::common_utils::{RoleGuard, UserRole};

#[derive(Default)]
pub struct AnalyticsQuery;

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
fn level_weight(level: CapabilityLevel) -> f64 {
    level_value(level) as f64 / 100.0
}

/// Collect all descendant org_tier IDs (including the given id) using a pre-loaded list of (id, parent_tier)
fn collect_descendant_tier_ids(root: Uuid, all_tiers_raw: &[(Uuid, Option<Uuid>)]) -> Vec<Uuid> {
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
fn get_person_ids_under_org_tier(org_tier_id: &Uuid) -> Result<Vec<Uuid>> {
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

#[Object]
impl AnalyticsQuery {
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
        let mut conn = connection()?;

        // Load capabilities: (id, domain, person_id, retired_at, self_identified_level, validated_level)
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

        let cap_rows: Vec<CapRow> = cap_query.load(&mut conn)?;

        // Load validations: (capability_id, validated_level, created_at)
        let val_rows: Vec<(Uuid, CapabilityLevel, NaiveDateTime)> = validations::table
            .select((
                validations::capability_id,
                validations::validated_level,
                validations::created_at,
            ))
            .load(&mut conn)?;

        // Build map: cap_id -> Vec<(created_at, level_value)>
        let mut val_map: HashMap<Uuid, Vec<(NaiveDateTime, i64)>> = HashMap::new();
        for (cap_id, level, created_at) in val_rows {
            val_map.entry(cap_id)
                .or_default()
                .push((created_at, level_value(level)));
        }

        // Build map: cap_id -> (domain, retired_at)
        let cap_meta: HashMap<Uuid, (SkillDomain, Option<NaiveDateTime>)> = cap_rows.iter()
            .map(|(id, d, _person_id, retired, _sil, _vl)| (*id, (*d, *retired)))
            .collect();

        // Determine time range
        let earliest_val = val_map.values()
            .flat_map(|v| v.iter().map(|(t, _)| *t))
            .min();

        let effective_from = from.or(earliest_val).unwrap_or_else(|| Utc::now().naive_utc());
        let effective_to = to.unwrap_or_else(|| Utc::now().naive_utc());

        let bucket_starts = bucket.generate_buckets(effective_from, effective_to);

        // Collect distinct domains from loaded data
        let mut domains: Vec<SkillDomain> = cap_rows.iter().map(|(_, d, _, _, _, _)| *d).collect();
        domains.sort_by_key(|d| format!("{:?}", d));
        domains.dedup();

        // If domain filter provided, narrow down
        let domains: Vec<SkillDomain> = if let Some(filter_domain) = domain {
            domains.into_iter().filter(|d| *d == filter_domain).collect()
        } else {
            domains
        };

        let mut result: Vec<LabeledSeries> = Vec::new();

        for d in domains {
            let mut points: Vec<TimeSeriesPoint> = Vec::new();
            // Caps in this domain
            let domain_caps: Vec<&(Uuid, SkillDomain, Uuid, Option<NaiveDateTime>, CapabilityLevel, Option<CapabilityLevel>)> =
                cap_rows.iter().filter(|(_, cap_d, _, _, _, _)| *cap_d == d).collect();

            for &period_start in &bucket_starts {
                let bend = bucket.bucket_end(period_start);
                let mut domain_sum = 0.0f64;

                for (cap_id, _, _, retired_at, _sil, _vl) in &domain_caps {
                    // Skip if retired before bucket start
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
                            domain_sum += avg / 100.0;
                        }
                    }
                }

                points.push(TimeSeriesPoint {
                    period_start,
                    bucket,
                    value: domain_sum,
                });
            }

            result.push(LabeledSeries {
                key: format!("{:?}", d),
                points,
            });
        }

        Ok(result)
    }

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
        let mut conn = connection()?;

        // ---- Supply: same as capability_growth ----
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

        let cap_rows: Vec<CapRow> = cap_query.load(&mut conn)?;

        let val_rows: Vec<(Uuid, CapabilityLevel, NaiveDateTime)> = validations::table
            .select((
                validations::capability_id,
                validations::validated_level,
                validations::created_at,
            ))
            .load(&mut conn)?;

        let mut val_map: HashMap<Uuid, Vec<(NaiveDateTime, i64)>> = HashMap::new();
        for (cap_id, level, created_at) in val_rows {
            val_map.entry(cap_id)
                .or_default()
                .push((created_at, level_value(level)));
        }

        // ---- Demand: requirements + works ----
        // Requirements: (id, domain, required_level, created_at, retired_at)
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
            // Filter via requirements -> roles -> teams -> org_tier descendants
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

        // Works: (id, domain, capability_level, created_at) — active only
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
        let earliest_val = val_map.values()
            .flat_map(|v| v.iter().map(|(t, _)| *t))
            .min();
        let effective_from = from.or(earliest_val).unwrap_or_else(|| Utc::now().naive_utc());
        let effective_to = to.unwrap_or_else(|| Utc::now().naive_utc());
        let bucket_starts = bucket.generate_buckets(effective_from, effective_to);

        // Collect domains
        let mut domains: Vec<SkillDomain> = cap_rows.iter().map(|(_, d, _, _, _, _)| *d)
            .chain(req_rows.iter().map(|(_, d, _, _, _)| *d))
            .chain(work_rows.iter().map(|(_, d, _, _)| *d))
            .collect();
        domains.sort_by_key(|d| format!("{:?}", d));
        domains.dedup();

        let domains: Vec<SkillDomain> = if let Some(filter_domain) = domain {
            domains.into_iter().filter(|d| *d == filter_domain).collect()
        } else {
            domains
        };

        let mut result: Vec<SupplyDemandSeries> = Vec::new();

        for d in domains {
            let domain_caps: Vec<_> = cap_rows.iter()
                .filter(|(_, cap_d, _, _, _, _)| *cap_d == d)
                .collect();
            let domain_reqs: Vec<_> = req_rows.iter()
                .filter(|(_, req_d, _, _, _)| *req_d == d)
                .collect();
            let domain_works: Vec<_> = work_rows.iter()
                .filter(|(_, wd, _, _)| *wd == d)
                .collect();

            let mut points: Vec<SupplyDemandPoint> = Vec::new();

            for &period_start in &bucket_starts {
                let bend = bucket.bucket_end(period_start);

                // Supply
                let mut supply = 0.0f64;
                for (cap_id, _, _, retired_at, _sil, _vl) in &domain_caps {
                    if let Some(r) = retired_at {
                        if *r < period_start {
                            continue;
                        }
                    }
                    if let Some(vals) = val_map.get(cap_id) {
                        let relevant: Vec<i64> = vals.iter()
                            .filter(|(cat, _)| *cat <= bend)
                            .map(|(_, v)| *v)
                            .collect();
                        if !relevant.is_empty() {
                            let avg = relevant.iter().sum::<i64>() as f64 / relevant.len() as f64;
                            supply += avg / 100.0;
                        }
                    }
                }

                // Demand from requirements
                let mut demand = 0.0f64;
                for (_id, _dom, req_level, req_created, req_retired) in &domain_reqs {
                    if *req_created > bend {
                        continue;
                    }
                    if let Some(r) = req_retired {
                        if *r <= period_start {
                            continue;
                        }
                    }
                    demand += level_weight(*req_level);
                }

                // Demand from works
                for (_id, _dom, cap_level, work_created) in &domain_works {
                    if *work_created <= bend {
                        demand += level_weight(*cap_level);
                    }
                }

                points.push(SupplyDemandPoint {
                    period_start,
                    bucket,
                    supply,
                    demand,
                });
            }

            result.push(SupplyDemandSeries {
                domain: format!("{:?}", d),
                points,
            });
        }

        Ok(result)
    }

    /// Capability depth per team across all skill domains (for a heatmap).
    #[graphql(guard = "RoleGuard::new(UserRole::User)")]
    pub async fn team_capability_matrix(
        &self,
        org_tier_id: Option<Uuid>,
    ) -> Result<Vec<TeamCapabilityRow>> {
        let mut conn = connection()?;

        // Get teams (filtered by org_tier descendants if provided)
        let team_rows: Vec<(Uuid, String)> = if let Some(tier_id) = org_tier_id {
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

        let mut result: Vec<TeamCapabilityRow> = Vec::new();

        for (team_id, team_name) in team_rows {
            // Get active occupied role person_ids
            let person_ids: Vec<Uuid> = roles::table
                .filter(roles::team_id.eq(team_id))
                .filter(roles::active.eq(true))
                .filter(roles::person_id.is_not_null())
                .select(roles::person_id)
                .load::<Option<Uuid>>(&mut conn)?
                .into_iter()
                .flatten()
                .collect();

            if person_ids.is_empty() {
                result.push(TeamCapabilityRow {
                    team_id,
                    team_name,
                    cells: vec![],
                });
                continue;
            }

            // Load capabilities for those persons (not retired)
            type CapRow2 = (SkillDomain, Uuid, Option<CapabilityLevel>, CapabilityLevel);
            let cap_rows: Vec<CapRow2> = capabilities::table
                .filter(capabilities::person_id.eq_any(&person_ids))
                .filter(capabilities::retired_at.is_null())
                .select((
                    capabilities::domain,
                    capabilities::person_id,
                    capabilities::validated_level,
                    capabilities::self_identified_level,
                ))
                .load::<CapRow2>(&mut conn)?;

            // Group by domain: depth = sum weights, people_count = distinct person_ids
            let mut domain_map: HashMap<String, (f64, std::collections::HashSet<Uuid>)> = HashMap::new();
            for (domain, person_id, validated_level, self_identified_level) in cap_rows {
                let weight = if let Some(vl) = validated_level {
                    level_weight(vl)
                } else {
                    level_weight(self_identified_level)
                };
                let entry = domain_map.entry(format!("{:?}", domain)).or_default();
                entry.0 += weight;
                entry.1.insert(person_id);
            }

            let cells: Vec<TeamCapabilityCell> = domain_map.into_iter()
                .filter(|(_, (depth, _))| *depth > 0.0)
                .map(|(domain, (depth, people))| TeamCapabilityCell {
                    domain,
                    depth,
                    people_count: people.len() as i32,
                })
                .collect();

            result.push(TeamCapabilityRow {
                team_id,
                team_name,
                cells,
            });
        }

        Ok(result)
    }

    /// Derived role transitions over a window, for mobility/promotion analysis.
    #[graphql(guard = "RoleGuard::new(UserRole::User)")]
    pub async fn talent_movements(
        &self,
        from: Option<NaiveDateTime>,
        to: Option<NaiveDateTime>,
        org_tier_id: Option<Uuid>,
    ) -> Result<Vec<TalentMovement>> {
        let mut conn = connection()?;

        // Load all occupied roles: (id, person_id, team_id, start_datestamp, rank, occupational_level)
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

        // Filter out any rows where person_id ended up None despite the IS NOT NULL filter
        type RoleRow = (Uuid, Uuid, Uuid, NaiveDateTime, Option<crate::models::Rank>, Option<i32>);
        let role_rows: Vec<RoleRow> = role_rows_raw.into_iter()
            .filter_map(|(id, pid, tid, start, rank, occ)| {
                pid.map(|p| (id, p, tid, start, rank, occ))
            })
            .collect();

        // Group by person_id
        let mut person_roles: HashMap<Uuid, Vec<(Uuid, Uuid, NaiveDateTime, Option<crate::models::Rank>, Option<i32>)>> = HashMap::new();
        for (role_id, person_id, team_id, start, rank, occ_level) in role_rows {
            person_roles.entry(person_id)
                .or_default()
                .push((role_id, team_id, start, rank, occ_level));
        }

        // Get team_ids under org_tier (if provided) for filtering
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

            // INFLOW at first role
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

            // Transitions between consecutive roles
            for i in 1..person_role_list.len() {
                let (_prev_id, prev_team, prev_start, prev_rank, prev_occ) = &person_role_list[i - 1];
                let (_curr_id, curr_team, curr_start, curr_rank, curr_occ) = &person_role_list[i];

                let kind = if let (Some(pr), Some(cr)) = (prev_rank, curr_rank) {
                    if rank_order(cr) > rank_order(pr) {
                        "PROMOTION"
                    } else {
                        "LATERAL"
                    }
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
