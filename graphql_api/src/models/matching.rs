use std::collections::{HashMap, HashSet};

use async_graphql::*;
use uuid::Uuid;

use crate::models::{Capability, CapabilityLevel, Person, Requirement};

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
}

/// Tiered match result for a role.
#[derive(SimpleObject)]
pub struct RoleMatchResult {
    pub role_id: Uuid,
    /// People who meet every requirement at or above the required level,
    /// sorted by match_score descending.
    pub full_matches: Vec<PersonMatchScore>,
    /// People who meet at least min_coverage of requirements and have no single
    /// skill gap exceeding max_gap_per_req, sorted by match_score descending.
    pub partial_matches: Vec<PersonMatchScore>,
}

// Each capability level is a significant leap, so each missing level costs
// 10 points out of 100 in the composite score.
const GAP_PENALTY: f64 = 0.10;

/// Scores a single person against all role requirements.
/// `caps_by_skill` maps skill_id → all active capabilities for that skill
/// (pre-built once per call to `find_fuzzy_matches`).
fn score_person(
    person_id: Uuid,
    requirements: &[Requirement],
    caps_by_skill: &HashMap<Uuid, Vec<&Capability>>,
) -> PersonMatchScore {
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

    let n = requirements.len() as i32;
    let coverage = met as f64 / n as f64;
    let match_score = (coverage - (total_gap as f64 * GAP_PENALTY)).max(0.0);

    PersonMatchScore {
        person: Person::get_by_id(&person_id).unwrap(),
        match_score,
        requirements_met: met,
        requirements_total: n,
        coverage,
        total_gap,
        requirement_gaps: gaps,
    }
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

    if requirements.is_empty() {
        return Ok(RoleMatchResult {
            role_id,
            full_matches: vec![],
            partial_matches: vec![],
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

    // Unique person_ids seen across all returned capabilities.
    let person_ids: HashSet<Uuid> = all_caps.iter().map(|c| c.person_id).collect();

    let mut full: Vec<PersonMatchScore> = Vec::new();
    let mut partial: Vec<PersonMatchScore> = Vec::new();

    for person_id in person_ids {
        let score = score_person(person_id, &requirements, &caps_by_skill);

        // Drop anyone with a single skill gap exceeding the caller's threshold.
        if score.requirement_gaps.iter().any(|g| g.gap > max_gap_per_req) {
            continue;
        }

        if score.coverage >= 1.0 {
            full.push(score);
        } else if score.coverage >= min_coverage {
            partial.push(score);
        }
    }

    full.sort_by(|a, b| b.match_score.partial_cmp(&a.match_score).unwrap());
    partial.sort_by(|a, b| b.match_score.partial_cmp(&a.match_score).unwrap());
    full.truncate(limit);
    partial.truncate(limit);

    Ok(RoleMatchResult {
        role_id,
        full_matches: full,
        partial_matches: partial,
    })
}
