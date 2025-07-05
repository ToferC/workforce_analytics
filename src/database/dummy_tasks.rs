use async_graphql::Error;
use rand::{seq::SliceRandom, rngs::ThreadRng, Rng};
use uuid::Uuid;

use crate::models::{NewTask, Task, SkillDomain, WorkStatus, NewRequirement, CapabilityLevel, Rank};

/// Generate dummy tasks based on some baseline data about the org
pub fn generate_tasks(
    rng: &mut ThreadRng,
    domain: &SkillDomain, 
    subject: &str, 
    creating_role_id: &Uuid,
    tier_level: i32,
) -> Result<Task, Error> {

    let work_nouns: Vec<&str> = "
        operations order; intelligence briefing; tactical assessment; situation report; 
        mission plan; reconnaissance report; training schedule; equipment manifest; 
        security protocol; maintenance log; deployment order; field manual; 
        threat analysis; combat readiness report; logistics plan; patrol route
    ".split("; ").collect();

    let outcome: Vec<&str> = "
        execute; coordinate; assess; secure; defend; 
        deploy; train; maintain; surveil; support; 
        command; protect; patrol; establish; monitor
    ".split("; ").collect();

    let title = format!("{} on {}", 
        work_nouns.choose(rng).unwrap().trim().to_string(),
        subject
    );

    let nt = NewTask::new(
        *creating_role_id,
        title,
        *domain,
        outcome.choose(rng).unwrap().to_string(),
        tier_level,
        "https://www.forces.gc.ca/some_url".to_string(),
        chrono::Utc::now().naive_utc(),
        chrono::Utc::now().naive_utc(),
        WorkStatus::InProgress,
    );

    let task = Task::create(&nt);

    task
}

/// Generate requirement for a role based on a provided skilldomain
pub fn generate_requirement(role_id: Uuid, skill_id: Uuid, rank: Rank, rng: &mut impl Rng) -> NewRequirement {
    // Add requirements for each role based on military rank structure

    let req_level: CapabilityLevel;

    // Two-tier system: Non-Commissioned Members (NCMs) and Officers
    let base_level = match rank {
        // Non-Commissioned Members (NCMs) - enlisted personnel
        Rank::Private => 1,                    // New recruits
        Rank::Corporal => 2,                   // Junior NCM
        Rank::MasterCorporal => 3,             // Senior NCM
        Rank::Sergeant => 5,                   // Staff NCM
        Rank::WarrantOfficer => 8,             // Senior NCM leader
        Rank::MasterWarrantOfficer => 10,       // Very experienced NCM
        Rank::ChiefWarrantOfficer => 12,        // Most experienced NCM
        
        // Officers - commissioned personnel (start at higher base level)
        Rank::SecondLieutenant => 5,           // New officer
        Rank::Lieutenant => 6,                 // Junior officer
        Rank::Captain => 7,                    // Company-level officer
        Rank::Major => 8,                      // Senior officer
        Rank::LieutenantColonel => 9,          // Battalion-level commander
        Rank::Colonel => 10,                   // Senior commander
        Rank::BrigadierGeneral => 11,          // General officer
        Rank::MajorGeneral => 12,              // Senior general
        Rank::LieutenantGeneral => 13,         // Very senior general
        Rank::General => 14,                   // Highest rank
    };

    // Allow for individual variation within rank expectations
    let adjusted_level = base_level + rng.gen_range(-1..=1);

    req_level = match adjusted_level {
        0..=2 => CapabilityLevel::Desired,     // New personnel
        3..=4 => CapabilityLevel::Novice,      // Basic competency
        5..=8 => CapabilityLevel::Experienced, // Standard military competency
        9..=12 => CapabilityLevel::Expert,     // Leadership positions
        13..=15 => CapabilityLevel::Specialist, // Strategic level
        _ => CapabilityLevel::Experienced,     // Default fallback
    };

    NewRequirement::new(
        role_id,
        skill_id,
        req_level,
    )
}