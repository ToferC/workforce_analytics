//! Hierarchy-scoped authorization.
//!
//! Role-based ownership (Phase 1) makes a Role the unit of organizational
//! authority: a Role that owns an OrgTier or Team is its manager and may manage
//! everything beneath that node. This module turns that data into an
//! enforcement check for guarded mutations.
//!
//! A principal may perform a scoped mutation on an entity iff **either**:
//!   * their global tier is `admin` (organization-wide authority), or
//!   * one of their active Roles owns an OrgTier or Team that is an
//!     ancestor-or-self of the entity's position in the hierarchy.
//!
//! Resolution path: `User -> Person -> active Role(s) -> owned OrgTiers/Teams
//! -> subtree`. Principals without a Person (admins and agent service accounts)
//! have an empty scope, so agents are naturally excluded from org-scoped
//! mutations unless they also hold the `admin` tier.

use std::collections::HashSet;

use async_graphql::{Context, Error, Result};
use diesel::prelude::*;
use uuid::Uuid;

use crate::common_utils::UserRole;
use crate::database::connection;
use crate::models::{Person, Role, Task, Work};
use crate::schema::{org_tier_ownerships, org_tiers, team_ownerships, teams};

/// Whether hierarchy-scoped authorization is enforced. Enabled by default; set
/// `DISABLE_SCOPED_AUTHZ=true` to fall back to flat operator+ authority during a
/// grandfather window while ownership data is being populated.
pub fn scoped_authz_enabled() -> bool {
    !matches!(
        std::env::var("DISABLE_SCOPED_AUTHZ").ok().as_deref(),
        Some("true") | Some("1")
    )
}

fn current_role(ctx: &Context<'_>) -> Option<UserRole> {
    ctx.data_opt::<UserRole>().copied()
}

fn current_user_id(ctx: &Context<'_>) -> Option<Uuid> {
    ctx.data_opt::<Uuid>().copied()
}

/// The org tiers and teams the current principal may manage.
pub struct EffectiveScope {
    pub is_admin: bool,
    /// Managed org tiers, expanded to include all descendant tiers.
    pub tier_ids: HashSet<Uuid>,
    /// Teams owned directly, plus every team sitting under a managed tier.
    pub team_ids: HashSet<Uuid>,
}

impl EffectiveScope {
    fn empty(is_admin: bool) -> Self {
        EffectiveScope {
            is_admin,
            tier_ids: HashSet::new(),
            team_ids: HashSet::new(),
        }
    }

    pub fn manages_tier(&self, tier_id: &Uuid) -> bool {
        self.is_admin || self.tier_ids.contains(tier_id)
    }

    pub fn manages_team(&self, team_id: &Uuid) -> bool {
        self.is_admin || self.team_ids.contains(team_id)
    }
}

/// Compute the subtree of org tiers and teams the current principal can manage.
pub fn effective_scope(ctx: &Context<'_>) -> Result<EffectiveScope> {
    if current_role(ctx) == Some(UserRole::Admin) {
        return Ok(EffectiveScope::empty(true));
    }

    // Resolve the principal's Person and the roles they currently occupy.
    // Agents and admins have no Person, so their scope is empty.
    let user_id = match current_user_id(ctx) {
        Some(id) => id,
        None => return Ok(EffectiveScope::empty(false)),
    };

    let person = match Person::get_by_user_id(&user_id) {
        Ok(p) => p,
        Err(_) => return Ok(EffectiveScope::empty(false)),
    };

    let role_ids: Vec<Uuid> = Role::get_current_for_person(&person.id)?
        .iter()
        .map(|r| r.id)
        .collect();

    if role_ids.is_empty() {
        return Ok(EffectiveScope::empty(false));
    }

    let mut conn = connection()?;

    // OrgTiers and Teams owned directly by the principal's active roles.
    let owned_tiers: Vec<Uuid> = org_tier_ownerships::table
        .filter(org_tier_ownerships::owner_role_id.eq_any(&role_ids))
        .select(org_tier_ownerships::org_tier_id)
        .load::<Uuid>(&mut conn)?;

    let owned_teams: Vec<Uuid> = team_ownerships::table
        .filter(team_ownerships::owner_role_id.eq_any(&role_ids))
        .select(team_ownerships::team_id)
        .load::<Uuid>(&mut conn)?;

    // Expand owned tiers to include every descendant tier.
    let all_tiers: Vec<(Uuid, Option<Uuid>)> = org_tiers::table
        .select((org_tiers::id, org_tiers::parent_tier))
        .load::<(Uuid, Option<Uuid>)>(&mut conn)?;

    let mut tier_ids: HashSet<Uuid> = HashSet::new();
    let mut queue: Vec<Uuid> = owned_tiers;
    while let Some(current) = queue.pop() {
        if tier_ids.insert(current) {
            for (tid, parent) in &all_tiers {
                if *parent == Some(current) {
                    queue.push(*tid);
                }
            }
        }
    }

    // Teams under any managed tier, plus directly owned teams.
    let tier_vec: Vec<Uuid> = tier_ids.iter().copied().collect();
    let mut team_ids: HashSet<Uuid> = teams::table
        .filter(teams::org_tier_id.eq_any(&tier_vec))
        .select(teams::id)
        .load::<Uuid>(&mut conn)?
        .into_iter()
        .collect();
    team_ids.extend(owned_teams);

    Ok(EffectiveScope {
        is_admin: false,
        tier_ids,
        team_ids,
    })
}

fn deny() -> Error {
    Error::new("Access denied: this entity is outside your managed organizational scope")
}

/// Require that the current principal manages `tier_id` (the tier itself or an
/// ancestor of it). Top-level tiers (no manager above them) are admin-only.
pub fn require_manage_tier(ctx: &Context<'_>, tier_id: &Uuid) -> Result<()> {
    if !scoped_authz_enabled() {
        return Ok(());
    }
    if effective_scope(ctx)?.manages_tier(tier_id) {
        Ok(())
    } else {
        Err(deny())
    }
}

/// Require that the current principal manages the tier a new/updated tier hangs
/// under. A tier with no parent is top-level and therefore admin-only.
pub fn require_manage_parent_tier(ctx: &Context<'_>, parent_tier: Option<&Uuid>) -> Result<()> {
    if !scoped_authz_enabled() {
        return Ok(());
    }
    match parent_tier {
        Some(parent) => require_manage_tier(ctx, parent),
        None => {
            if current_role(ctx) == Some(UserRole::Admin) {
                Ok(())
            } else {
                Err(Error::new(
                    "Access denied: only an admin may manage a top-level org tier",
                ))
            }
        }
    }
}

/// Require that the current principal manages `team_id`.
pub fn require_manage_team(ctx: &Context<'_>, team_id: &Uuid) -> Result<()> {
    if !scoped_authz_enabled() {
        return Ok(());
    }
    if effective_scope(ctx)?.manages_team(team_id) {
        Ok(())
    } else {
        Err(deny())
    }
}

/// Require that the current principal manages the team the given role sits on.
pub fn require_manage_role(ctx: &Context<'_>, role_id: &Uuid) -> Result<()> {
    if !scoped_authz_enabled() {
        return Ok(());
    }
    let role = Role::get_by_id(role_id)?;
    require_manage_team(ctx, &role.team_id)
}

/// Require that the current principal manages the team the task's creator role
/// sits on. Work is authorized through the task it belongs to.
pub fn require_manage_task(ctx: &Context<'_>, task_id: &Uuid) -> Result<()> {
    if !scoped_authz_enabled() {
        return Ok(());
    }
    let task = Task::get_by_id(task_id)?;
    require_manage_role(ctx, &task.created_by_role_id)
}

/// Whether the current principal currently occupies `role_id` (i.e. holds it as
/// one of their active roles). Used for the assignee side of Proposal 3.
fn occupies_role(ctx: &Context<'_>, role_id: &Uuid) -> bool {
    let user_id = match current_user_id(ctx) {
        Some(id) => id,
        None => return false,
    };
    let person = match Person::get_by_user_id(&user_id) {
        Ok(p) => p,
        Err(_) => return false,
    };
    Role::get_current_for_person(&person.id)
        .map(|roles| roles.iter().any(|r| r.id == *role_id))
        .unwrap_or(false)
}

/// Require that the caller may post a comment/flag on the given work
/// (Proposal 3, option (a)): allowed if they **manage** the owning task, or if
/// they currently **occupy the role the work is assigned to**. This is the one
/// place a below-operator user may write, and only against their own work.
pub fn require_comment_on_work(ctx: &Context<'_>, work_id: &Uuid) -> Result<()> {
    let work = Work::get_by_id(work_id)?;

    // Managers of the owning task may always comment.
    if require_manage_task(ctx, &work.task_id).is_ok() {
        return Ok(());
    }

    // Otherwise the caller must occupy the role this work is assigned to.
    match work.role_id {
        Some(role_id) if occupies_role(ctx, &role_id) => Ok(()),
        _ => Err(Error::new(
            "Access denied: you may only comment on work you manage or are assigned to",
        )),
    }
}
