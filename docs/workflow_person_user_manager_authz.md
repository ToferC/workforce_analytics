# Staged Parallel Workflow — Person-as-User & Manager-Scoped Authz

Coordination doc for implementing
[`roadmap_person_user_manager_authz.md`](./roadmap_person_user_manager_authz.md)
across the two repos. The roadmap is the *what*; this is the *how / who / when*.

## Tracks

| Track | Repo | Branch | Scope |
|---|---|---|---|
| **Backend** | `toferc/workforce_analytics` (`graphql_api/`) | `claude/vigilant-feynman-ow83pu` | Roadmap Phases 0–4 |
| **Frontend** | `toferc/workforce-frontend` (Rust/Tera/HTMX) | `claude/vigilant-feynman-ow83pu` | Roadmap Phase 5, adapted to Tera |

> **Frontend target decision:** the roadmap's Phase 5 contract was written for
> the React app in `workforce_analytics/orgchart`, but that directory is a stock
> Create-React-App skeleton. The live, maintained UI is the **`workforce-frontend`
> Rust/Tera/HTMX app**, which already consumes the API via `src/graphql/client.rs`
> + `queries/`. The frontend track therefore lands in `workforce-frontend`, with
> the contract adapted from React/TS to Tera templates + the existing client.

## Sequencing (the staging)

```
Backend:  Phase 0 ──> Phase 1 ──> Phase 2 (CONTRACT) ──> Phase 3 ──> Phase 4
                                       │
Frontend:                              └──> Phase 5 (parallel, gated on 2.3)
```

- **Phases 0 → 1 → 2 are sequential and blocking.** They establish the
  span-of-control primitives, the Person=User invariant, and caller identity in
  the resolver context.
- **Phase 2.3 is the synchronization point.** Once the `me { … managedTierIds }`
  query, the `redeemInvite` mutation, and the access-denied error semantics are
  merged (even as stubs), the frontend track unblocks and runs **in parallel**
  with backend Phases 3–4.
- Within a phase, independent checklist items can be parallelized across agents;
  across phases, respect the arrows.

### Gate conditions

| Stage | Unblocks when | Parallelism |
|---|---|---|
| S0 — Foundations | now | Backend only (Phase 0) |
| S1 — Person=User | Phase 0 merged | Backend only (Phase 1; 1.1 before 1.2) |
| S2 — Identity + Contract | Phase 1 merged | Backend only (Phase 2) |
| **S3 — Enforcement ‖ Frontend** | **Phase 2.3 merged** | Backend (3,4) **and** Frontend (5) in parallel |

## Status

| Phase | Item | Status | Notes |
|---|---|---|---|
| 0 | 0.1 lift `collect_descendant_tier_ids` → `models/authz.rs` | ✅ done | shared, `pub`; `analytics_query.rs` imports it |
| 0 | 0.2 `Person::managed_tier_ids` | ✅ done | direct + team-owned tiers + descendants |
| 0 | 0.3 ownership predicates | ✅ done | `can_manage_org_tier/team/role/person` |
| 0 | 0.4 span-of-control unit tests | ✅ done | pure-traversal tests in `authz.rs` (no DB) |
| 1 | 1.1–1.5 Person=User provisioning | ⬜ next | migrations + `create_person` transaction + dummy data |
| 2 | 2.1–2.3 caller identity + contract | ⬜ blocked on 1 | **publish contract at 2.3** |
| 3 | 3.1–3.5 `OwnershipGuard` + scoped mutations | ⬜ blocked on 2 | |
| 4 | 4.1–4.3 Product/Task/Work scoping | ⬜ blocked on 3 | |
| 5 | FE-1…FE-6 (adapted to Tera) | ⬜ blocked on 2.3 | in `workforce-frontend` |

## Phase 0 — what landed (this PR)

`graphql_api/src/models/authz.rs` (new shared module):

- `collect_descendant_tier_ids(root, &[(id, parent)])` — moved out of
  `analytics_query.rs` (which now imports it); behaviour preserved for its 5 call
  sites.
- `collect_descendant_tier_ids_many(roots, …)` — multi-root, de-duplicated,
  cycle-safe traversal.
- `managed_tier_ids_for_person(person_id)` — directly owned tiers
  (`org_tier_ownerships`) + team-owned tiers (`team_ownerships` → team → tier),
  expanded to all descendants.
- `can_manage_org_tier / can_manage_team / can_manage_role / can_manage_person`.
- `Person::managed_tier_ids` + the four `can_manage_*` methods delegate here.
- Unit tests cover direct owner, mid-level owner (no climb / no sibling),
  leaf owner, overlapping-root dedupe, and cycle termination.

**Deferred to later phases (noted for the next agent):**

- Per-request caching of `managed_tier_ids` → **Phase 2.2** (recomputed per call
  today; fine for Phase 0, the predicates each recompute the span).
- Admin-bypass lives in the **guard** (`OwnershipGuard`, Phase 3.1), not in these
  predicates — the predicates answer "does this person manage X?", the guard
  layers "Admin OR (Operator AND manages X)".

## Frontend contract (Tera adaptation, finalized at backend 2.3)

The React shapes in the roadmap map to `workforce-frontend` as:

- `me { user person managedTierIds isManager }` → add a query under
  `queries/` + a typed wrapper in `src/graphql/`; expose `managed_tier_ids` /
  `is_manager` through `generate_basic_context` so templates can gate.
- `canManage(tierId)` → a Tera-visible helper / context flag; hide Edit / Assign
  / Reparent / Change-owner controls for out-of-span entities (server still
  enforces — template checks hide buttons only, per `workforce-frontend/CLAUDE.md`).
- `redeemInvite(accessKey, password)` → an activation page + POST handler keyed
  off the `access_key` token (CSRF-validated like every mutating handler).
- Person create form drops the `userId` field (auto-provisioned by the API).
- Product-owner / task / work assignment pickers pre-filter to managed scope.
- Access-denied GraphQL errors surface as flash messages
  (`security::add_flash(session, "danger", …)`), not fatal errors.

## PRs

- Backend: draft PR on `claude/vigilant-feynman-ow83pu` → `main` in
  `workforce_analytics`, one PR per phase (or per stage) so review stays small.
- Frontend: draft PR on `claude/vigilant-feynman-ow83pu` → `main` in
  `workforce-frontend`, opened (skeleton) now so the track is visible; fills in
  at S3.

## Risks carried from the roadmap

- **1.1 backfill:** duplicate/empty Person emails collide with `users.email`
  UNIQUE — needs a dedup pass before the FK in 1.2.
- **4.1 product ownership:** validate role-holder-is-manager vs. add explicit
  `manager_person_id` — confirm with owner before building.
- **Invite delivery:** email out of scope; tokens surfaced to Admin/manager.
