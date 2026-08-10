# Roadmap: Person-as-User & Manager-Scoped Authorization

Status: **Proposed** · Branch: `claude/lucid-fermat-4hird7` · Owner: backend + frontend (parallel)

## Goal

Enable a system where:

1. **Every Person is a User** and can access the system (auto-provisioned, invite-based onboarding).
2. **Managers can modify the OrgTiers and Teams they are responsible for** — i.e. anything at or below them in the hierarchy — and assign **Tasks** and **Work** to those teams.
3. **Products are assigned to managers.**

A "manager" is defined as a **Person who owns an OrgTier or Team** (via `org_tier_ownerships` / `team_ownerships`). A manager's **span of control** is their owned tier(s) plus all descendant tiers, and the teams/roles/people within them.

### Confirmed design decisions

- **Cache `managed_tier_ids`** per request (computed once, stored in GraphQL context) rather than recomputing per guard check.
- **Auto-provision Users with an invite token** on Person creation (reuse the existing unused `users.access_key` column as the invite/reset token). People do not set a password at creation time.

---

## Architecture summary (current state)

| Concern | Where it lives today | Notes |
|---|---|---|
| Person | `graphql_api/src/models/person.rs:26` | `user_id UNIQUE NOT NULL`, but **no FK**; created separately from User |
| User | `graphql_api/src/models/user.rs:14` | `role`, `access_level`, unused `access_key` |
| Roles enum | `graphql_api/src/common_utils.rs:6` | `User < Analyst < Operator < Admin` |
| Auth guard | `graphql_api/src/common_utils.rs:25` (`RoleGuard`) | Global role only, **no ownership scoping** |
| JWT / context | `graphql_api/src/models/auth.rs` | Injects `UserRole` only; **not `person_id`** |
| OrgTier hierarchy | `graphql_api/src/models/org_tier.rs:15` | `parent_tier` self-reference; `owner()` climbs tree |
| Team | `graphql_api/src/models/team.rs:19` | `org_tier_id`; `owner()` falls back up the tier tree |
| Ownership | `org_ownership.rs`, `team_ownership.rs` | `OrgOwnership`→Person, `TeamOwnership`→Person |
| Span-of-control traversal | `graphql_api/src/graphql/query/analytics_query.rs:34` (`collect_descendant_tier_ids`) | **Reuse this** — lift to shared module |
| Product | `graphql_api/src/models/product.rs:16` | `product_owner_role_id` (a Role, not a Person) |
| Task | `graphql_api/src/models/task.rs:17` | `created_by_role_id`, `product_id` |
| Work | `graphql_api/src/models/work.rs:24` | `task_id`, `role_id` |

**The core gap:** authorization is global-role-only. Every mutation (`createTeam`, `updateTeam`, `createTask`, `createWork`, `createProduct`…) gates on `Operator` with no check that the caller owns the specific entity.

---

## Workstreams

Work is split so backend (this doc's primary track) and frontend can proceed in parallel. The **API contract** in Phase 2 is the synchronization point — once it is agreed and stubbed, frontend can build against it.

### Phase 0 — Foundations (backend, blocking)

- [ ] **0.1** Lift `collect_descendant_tier_ids` out of `analytics_query.rs` into a shared module (e.g. `models/org_tier.rs` or a new `authz.rs`) so models and guards can call it.
- [ ] **0.2** Add `Person::managed_tier_ids(&self) -> Vec<Uuid>`: tiers owned directly (`OrgOwnership` + `TeamOwnership`→team→tier) plus all descendants.
- [ ] **0.3** Add ownership predicates: `Person::can_manage_org_tier(tier_id)`, `can_manage_team(team_id)`, `can_manage_person(person_id)`, `can_manage_role(role_id)`.
- [ ] **0.4** Unit tests for span-of-control across a multi-level tier tree (direct owner, ancestor owner, unrelated tier, admin bypass).

### Phase 1 — Person = User (backend, blocking for onboarding)

- [ ] **1.1** Migration: backfill `User` rows for every `Person` whose `user_id` does not resolve to a real user. Use person email/name, `role = "USER"`, generate `access_key` invite token, no usable password hash.
- [ ] **1.2** Migration: add FK `persons.user_id → users.id` (keep UNIQUE). Must run **after** 1.1.
- [ ] **1.3** Rework `create_person` (`person_mutation.rs`) into a single transaction: create `User` (Argon2 hash of random secret, invite `access_key`, `role="USER"`) → create `Person` with that `user_id`. Remove caller-supplied `user_id` from `NewPerson`.
- [ ] **1.4** Invite/activation flow: mutation to redeem `access_key` → set password. (Token email delivery is out of scope; expose the token to Admin/manager for now.)
- [ ] **1.5** Fix dummy data (`dummy_org_data.rs:265`) to create real Users instead of random `user_id`s.

### Phase 2 — Caller identity in context (backend, blocking for Phase 3) — **API CONTRACT POINT**

- [ ] **2.1** Extend JWT handling in `auth.rs` so after decoding, the resolver context carries the caller's `person_id` (lookup `Person` by `user_id = claims.sub`).
- [ ] **2.2** Compute and cache `managed_tier_ids` **once per request**, store in `context.data` (a `HashSet<Uuid>`). Add `current_person(ctx)` and `managed_tiers(ctx)` helpers.
- [ ] **2.3** Publish the API contract addendum (see "Frontend contract" below): new `me { person { ... } managedTiers }` query, error semantics for unauthorized mutations, invite redemption mutation.

### Phase 3 — Manager-scoped authorization (backend)

- [ ] **3.1** Add `OwnershipGuard` next to `RoleGuard`. Effective rule per scoped mutation: **Admin OR (Operator AND caller manages the target subtree)**.
- [ ] **3.2** Apply to OrgTier mutations: `updateOrgTier`, reparenting, ownership changes → `can_manage_org_tier`.
- [ ] **3.3** Apply to Team mutations: `createTeam` (in managed tier), `updateTeam`, team ownership → `can_manage_team` / `can_manage_org_tier`.
- [ ] **3.4** Apply to role assignment (`role.rs:388` `assign_person`): target role's team must be in span.
- [ ] **3.5** Negative-path tests: manager editing outside span → access denied; admin bypass passes.

### Phase 4 — Products, Tasks, Work scoped to managers (backend)

- [ ] **4.1** Products → managers. Validate at assignment that `product_owner_role_id`'s holder is a manager within the caller's span (preferred over duplicating ownership). `createProduct`/`updateProduct` gate on managing the product's org subtree.
- [ ] **4.2** Tasks: `createTask`/`updateTask` — target product/team must be in span.
- [ ] **4.3** Work: `createWork` and assigning `role_id` — target role's team must be in span (`can_manage_team`).

### Phase 5 — Frontend (parallel track, see below)

Frontend can begin against the Phase 2 contract once 2.3 stubs are merged, even before Phases 3–4 enforcement lands.

---

## Frontend contract (for the parallel agent)

> The frontend lives in `/orgchart` (React + TypeScript). This section is the interface the frontend can build against. Treat shapes as **proposed** until Phase 2.3 is merged; field names may be tuned during implementation — track the GraphQL SDL as source of truth.

### New / changed queries

- **`me`** — returns the authenticated caller's identity and span of control:
  ```graphql
  query Me {
    me {
      user { id email role }
      person { id givenName familyName }
      managedTierIds      # tiers (and implicitly teams/people) this user can manage
      isManager           # true if managedTierIds is non-empty
    }
  }
  ```
  Use `managedTierIds` / `isManager` to drive **client-side gating**: show edit affordances only for tiers/teams within span. (Server still enforces — client gating is UX only.)

### New mutations

- **Person creation** no longer takes `userId` (auto-provisioned). Forms should drop that field.
- **Invite redemption**: `redeemInvite(accessKey: String!, password: String!)` → sets the new user's password. Build an activation screen keyed off the `access_key` token.

### Authorization UX

- Mutations outside the caller's span return an **access-denied GraphQL error** (string message, same channel as today's `RoleGuard` denials). Frontend should surface these gracefully (toast / inline) rather than treating them as fatal.
- Hide/disable "Edit", "Assign Task", "Assign Work", "Reparent", "Change owner" controls for entities not in `managedTierIds`.

### Product / Task / Work UI

- **Product owner** selector should be limited to managers within the editor's span (the API will reject others; the UI should pre-filter).
- **Assign Task / Assign Work** team & role pickers should be filtered to the manager's teams.

### Suggested frontend tasks

- [ ] FE-1 Add `me` query + auth/identity context provider exposing `managedTierIds` / `isManager`.
- [ ] FE-2 Span-aware gating helper `canManage(tierId)`; wire into OrgTier/Team edit controls.
- [ ] FE-3 Remove `userId` from person-creation forms.
- [ ] FE-4 Invite activation screen (`redeemInvite`).
- [ ] FE-5 Filter product-owner / task / work assignment pickers to managed scope.
- [ ] FE-6 Standardize access-denied error surfacing.

---

## Sequencing & dependencies

```
Phase 0 ─┐
Phase 1 ─┼─> Phase 2 (contract) ─> Phase 3 ─> Phase 4
         │                    └─> Phase 5 (frontend, parallel)
```

- **0, 1, 2 are blocking and sequential-ish** (1.1 before 1.2; 2 needs nothing from 1 except for `me.person`).
- **Frontend (Phase 5) unblocks at 2.3** and can develop against stubbed resolvers while 3–4 land.

## Risks / open questions

- **Backfill correctness (1.1):** persons with duplicate/empty emails will collide with the `users.email` UNIQUE constraint — needs a dedup/cleanup pass first.
- **Performance:** `managed_tier_ids` walks the tier tree; mitigated by per-request caching (2.2). Revisit if orgs get very deep.
- **Product ownership model (4.1):** validate role-holder-is-manager vs. add explicit `manager_person_id` to `products`. Leaning validate-role-holder to keep one source of truth — confirm before building.
- **Invite delivery:** email sending is out of scope; tokens surfaced to Admin/manager for now.

## Definition of done

- Every Person row resolves to a real User; FK enforced.
- A manager can edit only tiers/teams/products/tasks/work within their span; admins unrestricted; non-managers blocked.
- `me` query returns identity + `managedTierIds`; frontend gates on it.
- Tests cover span-of-control positive/negative paths and the person→user provisioning transaction.
