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

    // Group by skill_id for O(1) lookup during per-person scoring.
    let mut caps_by_skill: HashMap<Uuid, Vec<&Capability>> = HashMap::new();
    for cap in &all_caps {
        caps_by_skill.entry(cap.skill_id).or_default().push(cap);
    }

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

        // Drop anyone with a single skill gap exceeding the caller's threshold.
        if score.requirement_gaps.iter().any(|g| g.gap > max_gap_per_req) {
            continue;
        }

        let is_full = score.coverage >= 1.0;
        let qualifies = is_full || score.coverage >= min_coverage;
        if !qualifies {
            continue;
        }

        match (score.in_managed_scope, is_full) {
            (true, true) => managed_full.push(score),
            (true, false) => managed_partial.push(score),
            (false, true) => external_full.push(score),
            (false, false) => external_partial.push(score),
        }
    }

    let sort_and_cap = |v: &mut Vec<PersonMatchScore>| {
        v.sort_by(|a, b| b.match_score.partial_cmp(&a.match_score).unwrap());
        v.truncate(limit);
    };
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

    fn group<'a>(caps: &'a [Capability]) -> HashMap<Uuid, Vec<&'a Capability>> {
        let mut m: HashMap<Uuid, Vec<&Capability>> = HashMap::new();
        for c in caps {
            m.entry(c.skill_id).or_default().push(c);
        }
        m
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
}
