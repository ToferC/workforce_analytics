use std::collections::{HashMap, HashSet};

use async_graphql::*;
use uuid::Uuid;

use crate::models::{Capability, CapabilityLevel, OrgOwnership, OrgTier, Person, Requirement, Role, Team};
use crate::graphql::query::get_person_ids_under_org_tier;

/// Per-requirement breakdown for a single candidate.
#[derive(SimpleObject, Clone)]
pub struct RequirementMatch {
    pub skill_id: Uuid,
    pub skill_name: String,
    pub required_level: CapabilityLevel,
    /// None when the person holds no capability for this skill at all.
    pub actual_level: Option<CapabilityLevel>,
    /// required - actual (negative = over-qualified, 0 = exact, positive = shortfall).
    pub gap: i32,
    pub met: bool,
}

/// Contact details for the manager who owns a candidate's current team.
/// Populated for candidates who fall outside the hiring role's managed area, so
/// the requester knows whose permission is needed to move them.
#[derive(SimpleObject, Clone)]
pub struct ManagerContact {
    /// The owning (manager) Role for the candidate's current team.
    pub owner_role_id: Uuid,
    pub owner_role_title: String,
    /// The candidate's current team name.
    pub team_name: String,
    /// Manager's name, if the owning role is currently filled.
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

/// A scored candidate for a role.
#[derive(SimpleObject)]
pub struct PersonMatchScore {
    pub person: Person,
    /// Composite score in [0, 1]: coverage minus gap penalties.
    pub match_score: f64,
    pub requirements_met: i32,
    pub requirements_total: i32,
    /// requirements_met / requirements_total.
    pub coverage: f64,
    /// Sum of positive (shortfall) gaps only.
    pub total_gap: i32,
    pub requirement_gaps: Vec<RequirementMatch>,
    /// True if the candidate currently holds a role under the OrgTier owned by
    /// this role's area. The owner (and admins) can reassign these directly.
    pub in_managed_scope: bool,
    /// The candidate's manager contact, populated only when the candidate is
    /// outside the managed area (moving them needs their manager's agreement).
    pub manager: Option<ManagerContact>,
}

/// A scored vacant role for a person — the mirror of `PersonMatchScore`, from
/// the same scoring model.
#[derive(SimpleObject)]
pub struct RoleMatchScore {
    pub role: Role,
    /// Composite score in [0, 1]: coverage minus gap penalties.
    pub match_score: f64,
    pub requirements_met: i32,
    pub requirements_total: i32,
    /// requirements_met / requirements_total.
    pub coverage: f64,
    /// Sum of positive (shortfall) gaps only.
    pub total_gap: i32,
    pub requirement_gaps: Vec<RequirementMatch>,
}

/// Tiered match result for a role.
#[derive(SimpleObject)]
pub struct RoleMatchResult {
    pub role_id: Uuid,
    /// The OrgTier whose owner is responsible for this role (nearest tier with
    /// an ownership record, walking up from the role's team). `in_managed_scope`
    /// candidates sit under this tier. None if no owner is assigned anywhere up
    /// the chain.
    pub managed_org_tier_id: Option<Uuid>,
    /// Candidates who currently hold a role under the managed OrgTier — the
    /// owner/admin can reassign these internally. Sorted by match_score desc.
    pub managed_full_matches: Vec<PersonMatchScore>,
    /// Managed-area candidates meeting min_coverage but not every requirement.
    pub managed_partial_matches: Vec<PersonMatchScore>,
    /// Full matches outside the managed area; each carries `manager` contact.
    pub external_full_matches: Vec<PersonMatchScore>,
    /// Partial matches outside the managed area; each carries `manager` contact.
    pub external_partial_matches: Vec<PersonMatchScore>,
}

// Each capability level is a significant leap, so each missing level costs
// 10 points out of 100 in the composite score.
const GAP_PENALTY: f64 = 0.10;

/// Pure scoring pass for one person: per-requirement gap breakdown from the
/// pre-grouped capabilities (skill_id → all active capabilities for that
/// skill). Returns (gaps, requirements_met, total_gap). Split from
/// `score_person` so the scoring math is unit-testable without a database or
/// a full `Person` row.
fn score_requirements(
    person_id: Uuid,
    requirements: &[Requirement],
    caps_by_skill: &HashMap<Uuid, Vec<&Capability>>,
) -> (Vec<RequirementMatch>, i32, i32) {
    let mut gaps = Vec::with_capacity(requirements.len());
    let mut total_gap = 0i32;
    let mut met = 0i32;

    for req in requirements {
        // Take the highest available level for this person on this skill.
        // validated_level is authoritative; fall back to self_identified_level
        // for candidates who haven't been assessed yet so they appear in
        // partial results rather than being silently excluded.
        let best_level: Option<CapabilityLevel> = caps_by_skill
            .get(&req.skill_id)
            .and_then(|caps| {
                caps.iter()
                    .filter(|c| c.person_id == person_id)
                    .map(|c| c.validated_level.unwrap_or(c.self_identified_level))
                    .max_by_key(|l| l.as_int())
            });

        let gap = best_level
            .map(|l| req.required_level.as_int() - l.as_int())
            .unwrap_or_else(|| req.required_level.as_int()); // no capability = full gap

        let is_met = gap <= 0;
        if is_met {
            met += 1;
        }
        if gap > 0 {
            total_gap += gap;
        }

        gaps.push(RequirementMatch {
            skill_id: req.skill_id,
            skill_name: req.name_en.clone(),
            required_level: req.required_level,
            actual_level: best_level,
            gap,
            met: is_met,
        });
    }

    (gaps, met, total_gap)
}

/// Group capabilities by skill for O(1) lookup while scoring.
fn group_by_skill(caps: &[Capability]) -> HashMap<Uuid, Vec<&Capability>> {
    let mut grouped: HashMap<Uuid, Vec<&Capability>> = HashMap::new();
    for cap in caps {
        grouped.entry(cap.skill_id).or_default().push(cap);
    }
    grouped
}

/// Whether a scored match is returned: no single-skill gap above
/// `max_gap_per_req`, and full coverage or at least `min_coverage`.
fn qualifies(gaps: &[RequirementMatch], coverage: f64, min_coverage: f64, max_gap_per_req: i32) -> bool {
    gaps.iter().all(|g| g.gap <= max_gap_per_req) && (coverage >= 1.0 || coverage >= min_coverage)
}

/// Sort best score first and keep the top `limit`.
fn rank<T>(matches: &mut Vec<T>, score: impl Fn(&T) -> f64, limit: usize) {
    matches.sort_by(|a, b| score(b).total_cmp(&score(a)));
    matches.truncate(limit);
}

/// Composite score in [0, 1]: coverage minus a penalty per missing level.
fn composite_score(met: i32, total: i32, total_gap: i32) -> (f64, f64) {
    let coverage = met as f64 / total as f64;
    let match_score = (coverage - (total_gap as f64 * GAP_PENALTY)).max(0.0);
    (coverage, match_score)
}

fn score_person(
    person: Person,
    requirements: &[Requirement],
    caps_by_skill: &HashMap<Uuid, Vec<&Capability>>,
    managed_person_ids: &HashSet<Uuid>,
) -> PersonMatchScore {
    let person_id = person.id;
    let (gaps, met, total_gap) = score_requirements(person_id, requirements, caps_by_skill);
    let n = requirements.len() as i32;
    let (coverage, match_score) = composite_score(met, n, total_gap);

    PersonMatchScore {
        person,
        match_score,
        requirements_met: met,
        requirements_total: n,
        coverage,
        total_gap,
        requirement_gaps: gaps,
        in_managed_scope: managed_person_ids.contains(&person_id),
        manager: None,
    }
}

/// Walk up from a role's team to the nearest OrgTier that has an ownership
/// record — the tier whose owner is responsible for filling this role. Returns
/// the owned tier id, or None if no owner is assigned anywhere up the chain.
fn owned_tier_for_role(role_id: Uuid) -> Result<Option<Uuid>> {
    let role = Role::get_by_id(&role_id)?;
    let team = Team::get_by_id(&role.team_id)?;
    let mut tier = OrgTier::get_by_id(&team.org_tier_id)?;
    loop {
        if OrgOwnership::get_by_org_tier_id(&tier.id).is_ok() {
            return Ok(Some(tier.id));
        }
        match tier.parent_tier {
            Some(parent_id) => tier = OrgTier::get_by_id(&parent_id)?,
            None => return Ok(None),
        }
    }
}

/// Build the manager contact for a candidate: the owner of the team where they
/// currently hold a role. Returns None if the candidate holds no current role
/// or the team has no resolvable owner.
fn manager_contact_for_person(person_id: Uuid) -> Option<ManagerContact> {
    let role = Role::get_current_for_person(&person_id).ok()?.into_iter().next()?;
    let team = Team::get_by_id(&role.team_id).ok()?;
    let owner_role = Role::get_by_id(&team.owner_role_id().ok()?).ok()?;

    let manager = owner_role.person_id.and_then(|pid| Person::get_by_id(&pid).ok());

    Some(ManagerContact {
        owner_role_id: owner_role.id,
        owner_role_title: owner_role.title_en.clone(),
        team_name: team.name_en.clone(),
        name: manager.as_ref().map(|p| format!("{} {}", p.given_name, p.family_name)),
        email: manager.as_ref().map(|p| p.email.clone()),
        phone: manager.as_ref().map(|p| p.phone.clone()),
    })
}

/// Finds full and partial candidate matches for a role's requirements.
///
/// Issues exactly two DB queries regardless of how many requirements the role
/// has: one for requirements, one batched capability lookup for all required
/// skills. All scoring runs in Rust.
///
/// - `min_coverage`: minimum fraction of requirements that must be met (0.0–1.0)
/// - `max_gap_per_req`: maximum shortfall allowed for any single requirement;
///   candidates with a larger single-skill gap are excluded entirely
/// - `limit`: maximum results returned per tier
pub fn find_fuzzy_matches(
    role_id: Uuid,
    min_coverage: f64,
    max_gap_per_req: i32,
    limit: usize,
) -> Result<RoleMatchResult> {
    let requirements = Requirement::get_by_role_id(role_id)?;

    // Resolve the managed area: the OrgTier whose owner is responsible for this
    // role, and the set of people holding roles under that subtree.
    let managed_org_tier_id = owned_tier_for_role(role_id).unwrap_or(None);
    let managed_person_ids: HashSet<Uuid> = match managed_org_tier_id {
        Some(tier_id) => get_person_ids_under_org_tier(&tier_id)
            .unwrap_or_default()
            .into_iter()
            .collect(),
        None => HashSet::new(),
    };

    if requirements.is_empty() {
        return Ok(RoleMatchResult {
            role_id,
            managed_org_tier_id,
            managed_full_matches: vec![],
            managed_partial_matches: vec![],
            external_full_matches: vec![],
            external_partial_matches: vec![],
        });
    }

    let skill_ids: Vec<Uuid> = requirements.iter().map(|r| r.skill_id).collect();

    // Single batched query — one round-trip for all skills.
    let all_caps = Capability::get_active_by_skill_ids(&skill_ids)?;

    let caps_by_skill = group_by_skill(&all_caps);

    // Unique person_ids seen across all returned capabilities, fetched as
    // rows in one batch. A person deleted between the two queries simply
    // drops out of the candidate pool instead of panicking the scorer.
    let person_ids: Vec<Uuid> = all_caps
        .iter()
        .map(|c| c.person_id)
        .collect::<HashSet<Uuid>>()
        .into_iter()
        .collect();
    let candidates = Person::get_by_ids(&person_ids)?;

    // Separate candidates inside the managed area from those outside it. The
    // owner/admin can reassign managed candidates directly; external candidates
    // need their manager's agreement, so they carry contact details.
    let mut managed_full: Vec<PersonMatchScore> = Vec::new();
    let mut managed_partial: Vec<PersonMatchScore> = Vec::new();
    let mut external_full: Vec<PersonMatchScore> = Vec::new();
    let mut external_partial: Vec<PersonMatchScore> = Vec::new();

    for person in candidates {
        let score = score_person(person, &requirements, &caps_by_skill, &managed_person_ids);
        if !qualifies(&score.requirement_gaps, score.coverage, min_coverage, max_gap_per_req) {
            continue;
        }

        let is_full = score.coverage >= 1.0;

        match (score.in_managed_scope, is_full) {
            (true, true) => managed_full.push(score),
            (true, false) => managed_partial.push(score),
            (false, true) => external_full.push(score),
            (false, false) => external_partial.push(score),
        }
    }

    let sort_and_cap = |v: &mut Vec<PersonMatchScore>| rank(v, |s| s.match_score, limit);
    sort_and_cap(&mut managed_full);
    sort_and_cap(&mut managed_partial);
    sort_and_cap(&mut external_full);
    sort_and_cap(&mut external_partial);

    // Attach manager contact to the external candidates we're returning (after
    // truncation, so we only do this work for displayed rows).
    for score in external_full.iter_mut().chain(external_partial.iter_mut()) {
        score.manager = manager_contact_for_person(score.person.id);
    }

    Ok(RoleMatchResult {
        role_id,
        managed_org_tier_id,
        managed_full_matches: managed_full,
        managed_partial_matches: managed_partial,
        external_full_matches: external_full,
        external_partial_matches: external_partial,
    })
}

/// Scores `roles` against one person's capabilities (the reverse direction of
/// `find_fuzzy_matches`, same model), keeping those that qualify, best first.
/// Roles without requirements have nothing to score against and are skipped.
fn score_roles(
    person_id: Uuid,
    caps_by_skill: &HashMap<Uuid, Vec<&Capability>>,
    roles: Vec<Role>,
    reqs_by_role: &HashMap<Uuid, Vec<Requirement>>,
    min_coverage: f64,
    max_gap_per_req: i32,
    limit: usize,
) -> Vec<RoleMatchScore> {
    let mut matches: Vec<RoleMatchScore> = roles
        .into_iter()
        .filter_map(|role| {
            let requirements = reqs_by_role.get(&role.id).filter(|r| !r.is_empty())?;
            let (gaps, met, total_gap) = score_requirements(person_id, requirements, caps_by_skill);
            let n = requirements.len() as i32;
            let (coverage, match_score) = composite_score(met, n, total_gap);
            qualifies(&gaps, coverage, min_coverage, max_gap_per_req).then_some(RoleMatchScore {
                role,
                match_score,
                requirements_met: met,
                requirements_total: n,
                coverage,
                total_gap,
                requirement_gaps: gaps,
            })
        })
        .collect();
    rank(&mut matches, |m| m.match_score, limit);
    matches
}

/// Vacant, active roles scored against a person's active capabilities, best
/// first. Three queries regardless of how many roles are open: the person's
/// capabilities, the vacant roles, and their requirements in one batch.
pub fn find_role_matches(
    person_id: Uuid,
    min_coverage: f64,
    max_gap_per_req: i32,
    limit: usize,
) -> Result<Vec<RoleMatchScore>> {
    let caps: Vec<Capability> = Capability::get_by_person_id(person_id)?
        .into_iter()
        .filter(|c| c.retired_at.is_none())
        .collect();

    let roles = Role::get_vacant(i64::MAX)?;
    let role_ids: Vec<Uuid> = roles.iter().map(|r| r.id).collect();
    let mut reqs_by_role: HashMap<Uuid, Vec<Requirement>> = HashMap::new();
    for req in Requirement::get_by_role_ids(&role_ids)? {
        reqs_by_role.entry(req.role_id).or_default().push(req);
    }

    Ok(score_roles(person_id, &group_by_skill(&caps), roles, &reqs_by_role, min_coverage, max_gap_per_req, limit))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use crate::models::SkillDomain;

    fn now() -> NaiveDateTime {
        NaiveDateTime::parse_from_str("2026-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()
    }

    fn requirement(skill_id: Uuid, level: CapabilityLevel) -> Requirement {
        Requirement {
            id: Uuid::new_v4(),
            name_en: "Threat Analysis".into(),
            name_fr: "Analyse des menaces".into(),
            domain: SkillDomain::CyberSecurity,
            role_id: Uuid::new_v4(),
            skill_id,
            required_level: level,
            created_at: now(),
            updated_at: now(),
            retired_at: None,
        }
    }

    fn capability(
        person_id: Uuid,
        skill_id: Uuid,
        self_level: CapabilityLevel,
        validated: Option<CapabilityLevel>,
    ) -> Capability {
        Capability {
            id: Uuid::new_v4(),
            name_en: "Threat Analysis".into(),
            name_fr: "Analyse des menaces".into(),
            domain: SkillDomain::CyberSecurity,
            person_id,
            skill_id,
            organization_id: Uuid::new_v4(),
            self_identified_level: self_level,
            validated_level: validated,
            created_at: now(),
            updated_at: now(),
            retired_at: None,
            validated_by_id: None,
            validated_at: None,
        }
    }

    fn group(caps: &[Capability]) -> HashMap<Uuid, Vec<&Capability>> {
        group_by_skill(caps)
    }

    fn vacant_role() -> Role {
        Role {
            id: Uuid::new_v4(),
            person_id: None,
            team_id: Uuid::new_v4(),
            title_en: "Analyst".into(),
            title_fr: "Analyste".into(),
            effort: 1.0,
            active: true,
            military_occupation: None,
            rank: None,
            occupational_group: None,
            occupational_level: None,
            start_datestamp: now(),
            end_date: None,
            created_at: now(),
            updated_at: now(),
            reports_to: None,
            annual_salary_cents: None,
        }
    }

    #[test]
    fn validated_level_meets_requirement_exactly() {
        let (person, skill) = (Uuid::new_v4(), Uuid::new_v4());
        let caps = vec![capability(person, skill, CapabilityLevel::Novice, Some(CapabilityLevel::Expert))];
        let reqs = vec![requirement(skill, CapabilityLevel::Expert)];

        let (gaps, met, total_gap) = score_requirements(person, &reqs, &group(&caps));
        assert_eq!(met, 1);
        assert_eq!(total_gap, 0);
        assert!(gaps[0].met);
        assert_eq!(gaps[0].gap, 0);
        assert_eq!(gaps[0].actual_level, Some(CapabilityLevel::Expert));
    }

    #[test]
    fn self_identified_level_used_when_unvalidated() {
        let (person, skill) = (Uuid::new_v4(), Uuid::new_v4());
        let caps = vec![capability(person, skill, CapabilityLevel::Experienced, None)];
        let reqs = vec![requirement(skill, CapabilityLevel::Expert)];

        let (gaps, met, total_gap) = score_requirements(person, &reqs, &group(&caps));
        assert_eq!(met, 0);
        assert_eq!(total_gap, 1); // Expert - Experienced = one level short
        assert_eq!(gaps[0].actual_level, Some(CapabilityLevel::Experienced));
    }

    #[test]
    fn missing_capability_counts_as_full_gap() {
        let person = Uuid::new_v4();
        let reqs = vec![requirement(Uuid::new_v4(), CapabilityLevel::Expert)];

        let (gaps, met, total_gap) = score_requirements(person, &reqs, &HashMap::new());
        assert_eq!(met, 0);
        assert_eq!(total_gap, CapabilityLevel::Expert.as_int());
        assert_eq!(gaps[0].actual_level, None);
        assert!(!gaps[0].met);
    }

    #[test]
    fn highest_capability_for_skill_wins() {
        let (person, skill) = (Uuid::new_v4(), Uuid::new_v4());
        // Two capabilities for the same skill; the stronger one should count.
        let caps = vec![
            capability(person, skill, CapabilityLevel::Novice, None),
            capability(person, skill, CapabilityLevel::Novice, Some(CapabilityLevel::Specialist)),
        ];
        let reqs = vec![requirement(skill, CapabilityLevel::Expert)];

        let (gaps, met, _) = score_requirements(person, &reqs, &group(&caps));
        assert_eq!(met, 1);
        assert_eq!(gaps[0].actual_level, Some(CapabilityLevel::Specialist));
        assert!(gaps[0].gap < 0, "over-qualified gap should be negative");
    }

    #[test]
    fn other_peoples_capabilities_are_ignored() {
        let (person, other, skill) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
        let caps = vec![capability(other, skill, CapabilityLevel::Specialist, Some(CapabilityLevel::Specialist))];
        let reqs = vec![requirement(skill, CapabilityLevel::Novice)];

        let (_, met, total_gap) = score_requirements(person, &reqs, &group(&caps));
        assert_eq!(met, 0);
        assert_eq!(total_gap, CapabilityLevel::Novice.as_int());
    }

    #[test]
    fn composite_score_penalizes_each_missing_level() {
        // Full coverage, no gap: perfect score.
        assert_eq!(composite_score(2, 2, 0), (1.0, 1.0));
        // Half coverage, two levels short in total: 0.5 - 2*0.10 = 0.3.
        let (coverage, score) = composite_score(1, 2, 2);
        assert!((coverage - 0.5).abs() < 1e-9);
        assert!((score - 0.3).abs() < 1e-9);
        // Score floors at zero rather than going negative.
        let (_, floored) = composite_score(0, 4, 12);
        assert_eq!(floored, 0.0);
    }

    #[test]
    fn person_matches_rank_roles_by_score_and_prefer_validated_level() {
        let (person, skill_a, skill_b) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
        // Skill A: self-identified Specialist but validated only Experienced —
        // the validated level must win. Skill B: unvalidated Expert.
        let caps = vec![
            capability(person, skill_a, CapabilityLevel::Specialist, Some(CapabilityLevel::Experienced)),
            capability(person, skill_b, CapabilityLevel::Expert, None),
        ];
        let (exact, one_short, too_far, unscored) = (vacant_role(), vacant_role(), vacant_role(), vacant_role());
        let reqs_by_role = HashMap::from([
            // Met: validated Experienced + self-identified Expert.
            (exact.id, vec![requirement(skill_a, CapabilityLevel::Experienced), requirement(skill_b, CapabilityLevel::Expert)]),
            // One level short on A because the validated level counts, not the self one.
            (one_short.id, vec![requirement(skill_a, CapabilityLevel::Expert), requirement(skill_b, CapabilityLevel::Novice)]),
            // Two levels short on A: beyond max_gap_per_req = 1.
            (too_far.id, vec![requirement(skill_a, CapabilityLevel::Specialist)]),
        ]);
        let roles = vec![too_far.clone(), one_short.clone(), unscored.clone(), exact.clone()];

        let matches = score_roles(person, &group(&caps), roles, &reqs_by_role, 0.5, 1, 10);

        let ids: Vec<Uuid> = matches.iter().map(|m| m.role.id).collect();
        assert_eq!(ids, vec![exact.id, one_short.id]);
        assert_eq!(matches[0].match_score, 1.0);
        assert!((matches[1].match_score - 0.4).abs() < 1e-9); // 0.5 coverage - 1 level * 0.10
        assert_eq!(matches[1].requirement_gaps[0].actual_level, Some(CapabilityLevel::Experienced));

        let capped = score_roles(person, &group(&caps), vec![one_short, exact.clone()], &reqs_by_role, 0.5, 1, 1);
        assert_eq!(capped.iter().map(|m| m.role.id).collect::<Vec<_>>(), vec![exact.id]);
    }
}
