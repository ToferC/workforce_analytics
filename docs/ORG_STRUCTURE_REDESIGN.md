# Organizational Structure Redesign

Status: **In progress** — Step 1 (the `reports_to` reporting spine) landing now.

This doc covers a two-part change to how we model and show organizational
structure: a **structural** change in the API (`workforce_analytics`) and a
**visual** change in the frontend (`workforce-frontend`). The same file is
committed to both repos.

## Motivation

`OrgTier` is useful as *scaffolding* — it clarifies ownership, authority, and
how teams and roles hang together inside an organization. But it is not how
people actually think about an organization. People think in **teams** and
**reporting relationships** ("who do I report to?", "who reports to me?").

Two concrete problems with the current model:

1. **Teams are purely horizontal.** Every role on a team implicitly reports to
   the team owner. In reality a team often has internal sub-hierarchies (team
   leads reporting to a manager) that are real but not worth standing up a whole
   new `OrgTier` for.
2. **Cross-organization relationships are implicit.** Authority is expressed
   only through tier/team ownership, so the actual reporting line between two
   positions in different teams or tiers has to be *inferred* from the tier
   tree rather than read directly.

### Tier semantics (domain context)

Levels are a meaningful seniority band tied to portfolio and rank, not just a
depth in a tree:

| Level | Civilian            | Military             | Responsibility                              |
|-------|---------------------|----------------------|---------------------------------------------|
| L0    | Deputy Minister     | Chief of Defence Staff | Whole-of-organization                     |
| L1    | Assistant Deputy Minister | Lieutenant-General | Major portfolio / branch               |
| L2    | Director General    | Brigadier-General    | Portfolio, leadership, org outcomes         |
| L3    | Director            | Colonel              | Portfolio, leadership, org outcomes         |
| L4    | Manager             | (manager)            | **Managers with employees — working teams** |

L0–L3 are leadership levels responsible for portfolios and outcomes; the "team"
at those levels is a small **leadership team**. **L4 is where the actual working
teams live.** This is the key insight that drives the visual change.

## The redundancy we're working around

The codebase already encodes "who is above whom" in several places:

- `org_tiers.parent_tier` — the tier scaffolding tree.
- `teams.org_tier_id` — teams hang off a tier.
- `org_tier_ownerships` / `team_ownerships` — a **Role** owns a tier/team
  (durable, position-based — the right grain).
- `roles.team_id` — roles sit flat in a team, implicitly reporting to the owner.

The dead `ReportingRelationship` model (`models/reporting_relationship.rs`) was
an old **person→person** reporting table. Its table isn't in `schema.rs`, it
references a non-existent `family_name` column, and nothing imports it. It is
superseded by `reports_to` and is being retired.

## Decisions

1. **Add `roles.reports_to: Option<Uuid>` (FK → roles) as the authoritative
   reporting edge.** Position→position, not person→person — consistent with the
   existing "a Role is a durable position; RoleAssignment is tenure" design.
2. **`reports_to` is nullable and falls back to the team owner.** A `NULL`
   `reports_to` means "reports to my team's owner role," mirroring the
   ancestor-walk already in `OrgTier::owner()`. This makes the feature
   backwards-compatible: existing roles need no manual wiring, and the migration
   backfills the implied edges.
3. **`reports_to` may cross teams and tiers.** A team-lead role reports to the
   manager role on the *same* team; an L3 Director's role reports to an L2 DG's
   role on a *different* team. No same-team constraint.
4. **The reporting graph must stay acyclic.** Self-reference is rejected and
   cycles are rejected on write (walk up the chain; if we reach the role being
   edited, refuse).
5. **`OrgTier` is demoted from load-bearing hierarchy to portfolio label.**
   Once `reports_to` exists, the org chart *is* the reporting graph. `OrgTier`
   keeps its name, `primary_domain`, and `tier_level` (a seniority band), but is
   no longer the thing that defines who reports to whom.
6. **Direction (not now): unify `OrgTier` + `Team` into one `OrgUnit`** with a
   `kind` (leadership | working) and a level. The visual merge below is a strong
   hint that these are the same concept at different levels. Deferred until the
   `reports_to` spine proves itself.

## Implementation plan

### Step 1 — API: the `reports_to` spine (this change)

`workforce_analytics`:

- **Migration** `…_add_role_reports_to`:
  - `ALTER TABLE roles ADD COLUMN reports_to UUID REFERENCES roles(id) ON DELETE SET NULL;`
    (`SET NULL` so removing a manager position leaves reports dangling-to-owner
    rather than cascading deletes through the org.)
  - Index on `roles(reports_to)` for `directReports` lookups.
  - Backfill: every non-owner role → its team's owner role; team-owner roles →
    their tier's owner role (both guarded against self-reference).
- **`schema.rs`**: add `reports_to -> Nullable<Uuid>` to the `roles` table
  (appended last to match `Queryable` field order).
- **`models/role.rs`**:
  - Add `reports_to: Option<Uuid>` to `Role` (appended last) and to `NewRole`
    (defaulted to `None` in `NewRole::new` so existing callers are unaffected).
  - Resolvers: `reportsTo: Role` (the explicit manager position, if any),
    `reportsToId: UUID`, `directReports: [Role!]!`, and `manager: Role` (the
    *effective* manager — explicit `reports_to`, else the team owner role).
  - `Role::set_reports_to(role_id, manager)` with self/cycle validation;
    `Role::get_direct_reports(role_id)`.
- **Mutation** `setRoleReportsTo(roleId, reportsToRoleId)` (operator-guarded,
  scoped authz via `require_manage_role`), plus accept `reports_to` on
  `createRole` (`NewRole`).
- Retire the dead `ReportingRelationship` model.
- Update `schema.graphqls` to match.

### Step 2 — Frontend: explorer + sync (next)

`workforce-frontend`:

- Sync `schema.graphql` with the new `Role` fields; add `reportsToId` /
  `directReports` (and `manager`) to the explorer's team-member query.
- **Draw connectors from `reports_to`, not tier nesting** — intra-team
  sub-hierarchies (team leads under a manager) then appear automatically.
- **Merge tier + leadership-team into one box.** At L0–L3 the box folds in the
  tier's owner role and leadership-team roles; at L4 the box is the working team
  that expands into roles/people (current Phase 3 drill-down).
- **Level swimlanes / left rail (L0…L4)** so seniority reads at a glance.
- Visually distinguish leadership vs working units.
- A `reports_to` editing affordance can come from the org-chart *builder* later.

### Step 3 — Later

Evaluate unifying `OrgTier` + `Team` into a single `OrgUnit` concept.

## Risks / notes

- **Cycles**: enforced on write; the migration backfill is acyclic by
  construction (members→owner, owner→tier-owner, both self-guarded, tier tree is
  a DAG).
- **Vacant managers**: `reports_to` points to a position, which may be vacant —
  intended. The `manager` resolver still resolves the position.
- **Two truths drifting**: ownership (`team_ownerships`) and `reports_to` can
  disagree. For now `reports_to = NULL` *derives* from the owner, so they agree
  by default; an explicit edit is a deliberate override. A later cleanup can make
  ownership fully derived from `reports_to`.
</content>
</invoke>
