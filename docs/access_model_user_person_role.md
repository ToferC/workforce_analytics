# Design: Standardizing the User / Person / Role Access Model

**Status:** Proposed
**Scope:** `workforce_analytics` (GraphQL API) and `workforce-frontend` (Actix/Tera UI)
**Author:** Design doc for review — no code changes yet.

## 1. Purpose

We use three models to control access, describe people, and describe how people
move through the organization. Their responsibilities have drifted and the
ownership/authority logic is inconsistent. This document defines the target
model and a phased plan to standardize it.

The two concrete defects this corrects:

1. **`OrgOwnership` (and `TeamOwnership`) point at a `Person`, not a `Role`.**
   When a person changes jobs, the responsibility for the org tier / team
   incorrectly travels with the *individual* instead of staying with the
   *position*.
2. **There is no hierarchy-scoped authorization.** Any `operator`+ user can edit
   any OrgTier, Team, Task, or Work anywhere in the organization. The business
   rule "managers and executives can change the things below them" is not
   implemented.

## 2. The three models

### 2.1 User — authentication and global access tier

`User` is the authentication identity. It carries a coarse, organization-wide
access tier used for field-level visibility and baseline mutation gating:

```
user < analyst < operator < admin
```

(`UserRole` in `graphql_api/src/common_utils.rs`; mirrored as `MinimumRole` in
the frontend's `src/security.rs`.)

**Invariant — User ↔ Person association:**

- A **human** `User` *must* be associated with exactly one `Person`.
- **Exceptions:** `admin` users and **agent** users (see §2.4) may exist
  **without** a `Person`. An admin is a platform operator who is not necessarily
  a modeled member of the organization; an agent is a non-human principal.

The association is stored today as `persons.user_id` (`NOT NULL UNIQUE → users.id`),
which enforces "a Person has a User" but not "a User has a Person." We keep that
FK and add an **application-level guarantee + health check** that every
*non-admin, non-agent* User resolves to a Person, rather than a second circular
FK.

### 2.2 Person — the individual

`Person` is the real-world individual: contact details, organization
affiliation, capabilities, publications, and career history. A Person occupies
Roles over time through the `role_assignments` join table (one active assignment
per person, enforced transactionally in `Role::assign_person`).

A Person is **not** an authority-bearing entity on its own. All authority flows
through the Role(s) the Person currently occupies.

### 2.3 Role — the position and the unit of authority

A `Role` is a formal or informal position in the organization. Roles are stable;
the Person filling a Role changes over time, the Role persists. A Role:

- belongs to a `Team` (which sits in an `OrgTier`);
- may be **vacant** (`person_id`/active assignment is null);
- may **own / manage** a `Team` and/or an `OrgTier`;
- is the target of `Task` creation, `Work` assignment, and `Product` ownership.

**Definition — "manager Role":** a Role that owns at least one Team or OrgTier.
This is **derived**, not a stored flag, so there is a single source of truth and
it survives reassignment of the person automatically. Owning a node *is* being
its manager, and grants authority over everything beneath that node.

### 2.4 Agent — non-human service principal (new)

An **agent** is a `User` that represents an application or data service querying
the API on its own behalf (ETL jobs, dashboards, integrations), not a human.

- Authenticates via a long-lived credential (the existing `access_key` /
  bearer token path), **not** an interactive password login.
- Has **no `Person`** and therefore **no organizational scope**; it operates
  purely on its global access tier.
- Is explicitly identifiable so it can be audited, rate-limited, and revoked
  independently of human accounts.

**Modeling:** add a principal/account-type discriminator to `User`:

```
account_type: HUMAN | AGENT      # default HUMAN
```

(Implemented as a Diesel-backed enum column, e.g. `users.account_type text NOT
NULL DEFAULT 'HUMAN'`.) `admin` and `agent` accounts are the two cases allowed
to lack a Person. An agent is typically provisioned at `analyst` (read) or a
purpose-built tier; agents should **not** be granted org-scoped mutation rights
because they have no Role from which to derive scope.

## 3. Authority model

A user may perform a scoped mutation on an entity iff **either**:

- their global tier is `admin` (organization-wide authority), **or**
- one of their **active Roles** owns an `OrgTier` or `Team` that is an
  **ancestor-or-self** of the entity's position in the hierarchy.

Resolution path:

```
User → Person → active Role(s) → owned OrgTiers/Teams → subtree (descendant tiers,
their teams, the roles on those teams, and the tasks/work/products attached to them)
```

This reuses the org-tier parent-chain walk and owner-inheritance the API already
has. Agents and Persons with no active Role have an empty scope (admin excepted).

## 4. Current vs. target data model

| Concept | Current reference | Target reference |
| --- | --- | --- |
| OrgTier owner | `org_tier_ownerships.owner_id → persons` | `org_tier_ownerships.owner_role_id → roles` |
| Team owner | `team_ownerships.person_id → persons` | `team_ownerships.owner_role_id → roles` |
| Product owner | `products.product_owner_role_id → roles` | unchanged ✅ |
| Task creator | `tasks.created_by_role_id → roles` | unchanged ✅ |
| Work assignee | `works.role_id → roles` (nullable) | unchanged ✅ |
| User account kind | (none) | `users.account_type` HUMAN/AGENT |

After this change, **all** ownership and assignment is Role-based and uniform.

## 5. Proposed changes by repository

### 5.1 `workforce_analytics` (API)

**Data model / migrations**
- Migrate `org_tier_ownerships.owner_id` → `owner_role_id UUID → roles(id)`.
- Migrate `team_ownerships.person_id` → `owner_role_id UUID → roles(id)`
  (keep the temporal `start_datestamp` / `end_date` columns).
- Add `users.account_type` (`HUMAN` default, `AGENT`).
- **Backfill:** for each existing ownership row, map the owning Person to their
  *active* Role (`role_assignments WHERE end_date IS NULL`). Emit a report for
  rows where the Person has **0 or >1** active roles; resolve those manually
  rather than guessing.

**Models / resolvers**
- Update `OrgOwnership` / `TeamOwnership` structs and the `OrgTier::owner()` /
  `Team` owner resolvers (including the inheritance walk) to return the owning
  **Role** (exposing `role.person` for display, "Vacant" when null).
- Move `ownedTeams` / `ownedOrgTiers` onto `Role`; keep aggregating passthroughs
  on `Person` (union across the person's active roles) so existing queries work.

**Authorization**
- New module `graphql_api/src/graphql/authz.rs`: `effective_scope(user)` →
  managed OrgTier subtrees + Teams; `can_manage(user, entity)`.
- Upgrade OrgTier / Team / Task / Work / Product **mutation** guards from
  "operator+" to "operator+ **and in scope** (or admin)". Field-level
  `RoleGuard` on sensitive columns is unchanged.
- Validate `product_owner_role_id` is a manager Role at/above the product's level.

**Agent support**
- Recognize `account_type = AGENT` in the auth path; allow Person-less tokens;
  tag the request context so agents are audited and excluded from org-scoped
  mutations.

### 5.2 `workforce-frontend`

- Re-sync `schema.graphql` with the API's `schema.graphqls`.
- Rewrite ownership queries (`create/update_org_ownership`,
  `create/update_team_ownership`, `org_ownership_by_tier_id`, etc.) to use
  `ownerRoleId`.
- Replace the **person-name text-input** owner forms (`org_tier.rs` /
  `team.rs` `assign_*_owner_post`, `resolve_person_by_name`) with a **Role
  dropdown**, reusing the `role_options()` pattern already in `product.rs`
  ("Person — Title (Team)" / "Vacant — Title (Team)").
- Owner-display templates render `owner.person` with a "Vacant" fallback.
- Add a scope helper so edit/assign controls are hidden when the user cannot
  manage the node. **Cosmetic only — the API remains the authoritative gate.**

## 6. Phasing

1. **Phase 1 — API data model:** ownership → Role migration + backfill,
   `users.account_type`, struct/resolver updates. Ships the named fix.
2. **Phase 2 — API authorization:** scoped `can_manage` + mutation guards;
   agent principal handling in the auth path.
3. **Phase 3 — Frontend:** schema sync, Role-based owner UI, control gating.

The API ships first in each phase; the frontend follows once `schema.graphqls`
is regenerated. Phases 1 and 3's schema changes must land together to avoid
breaking the generated client.

## 7. Risks and mitigations

- **Backfill ambiguity** (people with 0/>1 active roles): report and resolve
  manually; do not auto-assign.
- **Scope lockout**: tightening guards could lock out current operators who own
  nothing yet. Mitigation: ship Phase 2 behind a flag, or grandfather
  operator-global mutation rights until ownership data is populated.
- **Schema drift** between repos: land API + frontend schema changes together;
  `schema.graphql` (frontend) must mirror `schema.graphqls` (API).
- **Agent over-privilege**: agents have no Role-derived scope, so never grant
  them org-scoped mutation authority; keep them to read/global tiers and audit
  their usage.

## 8. Open decisions

- Should `team_ownerships` keep temporal history, or align with the
  retire-based model used by `org_tier_ownerships`? (Recommend: keep temporal.)
- Agent default tier and whether agents need a dedicated `AGENT` value in
  `UserRole` vs. reusing existing tiers + `account_type`. (Recommend:
  `account_type` discriminator + existing tiers.)
- Phase 2 rollout: feature flag vs. grandfather window.
