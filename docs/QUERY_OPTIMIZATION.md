# Query Optimization Review

A review of the data-access patterns across the **API** (`workforce_analytics`,
this repo) and the **frontend** (`workforce-frontend`), with prioritized
recommendations. Items marked ✅ are addressed in the accompanying "quick wins"
change; the rest are tracked here for follow-up.

## Headline

The API resolves nested GraphQL fields with **per-row database lookups and no
batching**, against a **default-sized connection pool**, and the **hottest
foreign keys were unindexed**. The frontend compounds this by **fetching whole
tables and filtering/paginating client-side**, and by requesting nested fields
that each fan out into N+1 on the server. A single team page (`teamByID`) with
~20 roles realistically issues **150–200 separate SQL queries**, each checking
out and returning a pooled connection.

### Worked example: one `teamByID` request

For a team with `R` roles, each occupied role with ~`W` work items:

| Field | Queries |
|---|---|
| `occupiedRoles` / `vacantRoles` | 1 each |
| `person` per role | `R` (`Person::get_by_id`) |
| `person.capabilities` per person | `R` |
| `work` per role | `R` (`Work::get_by_role_id`) |
| `task` per work item | `R·W` |
| `task.product` per task | `R·W` |
| team-level: `organization`, `organizationLevel`, `owner`, `capabilityCounts`, `headcount`, `totalEffort` | ~7 |

At `R = 20`, `W = 3` that is roughly **180 queries** for one page load.

---

## Backend recommendations

### B1 — Indexes on the most-traversed foreign keys ✅
`migrations/` defined 26 indexes, but the columns hit hardest by resolver
fan-out had none. Each unindexed lookup was a sequential scan, multiplied by the
N+1 pattern. Added in migration `2026-06-30-000002_add_hot_fk_indexes`:

| Column | Used by |
|---|---|
| `roles(team_id)` | `Team::occupied_roles / vacant_roles / roles / headcount / capability_counts` |
| `roles(person_id)` | headcount, capability_counts |
| `works(role_id)` | `Role::work`, `Role::effort` |
| `works(task_id)` | `Task::work`, `Product::work` |
| `tasks(product_id)` | `Product::tasks` |
| `tasks(created_by_role_id)` | task ownership / team task lists |
| `teams(org_tier_id)` | `Team::get_by_org_tier_id` (org chart) |

Created as plain `CREATE INDEX IF NOT EXISTS` to stay inside the embedded
migration transaction. If these tables grow large, recreate the index as
`CREATE INDEX CONCURRENTLY` in a standalone, non-transactional migration to
avoid write locks on a live database.

### B2 — DataLoader batching (N+1 root cause) ✅
Field resolvers called single-id getters per parent row — `Role::person` →
`Person::get_by_id`, `Role::work` → `Work::get_by_role_id`, `Work::task`,
`Work::role`, `Task::product`, `Team::owner` — turning a list of N parents into
N+ queries.

`async_graphql::dataloader::DataLoader`s now batch these edges
(`graphql/loaders.rs`): `PersonLoader`, `TeamLoader`, `RoleLoader`,
`TaskLoader`, `ProductLoader` (keyed by id), and `WorkByRoleLoader` (one-to-many
by `role_id`). Each collapses N lookups into a single `WHERE id = ANY($1)` /
`WHERE role_id = ANY($1)`, backed by the batched `get_by_ids` / `get_by_role_ids`
model methods. The B1 indexes keep those `ANY()` scans fast.

The loaders are registered **per request** in `handlers::endpoints::graphql`, so
their cache never outlives a single request — a schema-global loader would serve
stale rows after a mutation. The hot `team → role → person / work → task →
product` chain that previously cost ~180 queries now batches to a handful.

Follow-up: extend loaders to the remaining single-id edges
(`RoleAssignment::person/role`, `Role::manager/reports_to`,
`Person::capabilities` as a by-person-id list loader) as those views warrant.

### B3 — Connection pool sizing & robustness ✅ (partial)
- The pool was built with `PostgresPool::new(manager)` and **no `max_size`**,
  taking r2d2's default of 10. With per-resolver checkout, one nested page can
  momentarily need dozens of connections, so concurrent users hit the 30s
  checkout timeout. Now built via `Pool::builder().max_size(...)`, default 20,
  overridable with `DB_POOL_MAX_SIZE` to match Postgres `max_connections`. ✅
- `Role::person` used `Person::get_by_id(&p).unwrap()`, which **panics the
  worker** on any DB error and fails the whole request. Changed to `.ok()` so it
  resolves to `None` instead. ✅
- Follow-up: once DataLoaders land (B2), a single request-scoped connection
  becomes viable, further reducing checkout churn. Audit the remaining
  `.unwrap()`/`.expect()` calls in resolver paths for the same panic risk.

### B4 — Query depth / complexity limits ✅
`Schema::build(...)` previously set neither `.limit_depth()` nor
`.limit_complexity()`, so a deeply nested or huge query could amplify the
resolvers' per-field work without bound. The schema now applies
`.limit_depth(15)` and `.limit_complexity(1000)` (`graphql/utilities.rs`). The
deepest legitimate client query nests ~6 levels, so 15 leaves generous headroom
while blocking pathological recursion (e.g. `role → manager → team → owner`
cycles). Both are overridable via `GRAPHQL_MAX_DEPTH` / `GRAPHQL_MAX_COMPLEXITY`
so a genuinely larger query can be unblocked without a redeploy.

---

## Frontend recommendations

### F1 — Stop fetching whole tables to filter/paginate client-side
- `team_index` loads `all_teams`, then filters by query/retired and caps at
  `INDEX_PAGE_CAP` **in Rust**.
- `work_index` loads `all_work` and filters status/unassigned client-side;
  `vacancies` loads `all_work` and keeps `role.is_none()`.
- Dropdowns load entire tables: `role_options` = `all_roles`,
  `product_options` = `all_products`, `skill_options` = `all_skills`, and the
  work form's task picker = `all_tasks`.

Each transfers and resolves every row (and every row's nested N+1) to show one
page or one `<select>`. Add server-side filter/limit arguments, and use
lightweight `{id, label}`-only queries for pickers.

### F2 — Trim over-fetched nested fields in list queries
- `all_roles` pulls `person` and `team { organization }` for every role — 2–3
  extra per-row lookups × all roles.
- `all_products` requests `effort`, a computed aggregate that sums each
  product's work server-side.

Request only the fields the list view renders; defer expensive
computed/nested fields to detail views.

### F3 — Parallelize independent round-trips in handlers
Several handlers chain `await`s that have no data dependency — e.g.
`assign_work_form` does `get_work_by_id` → `get_me` → `all_roles` →
`skill_options` serially. Use `futures::try_join!` to run independent calls
concurrently so latency is the slowest call, not the sum. `role.rs`, `work.rs`,
and `person.rs` have the most call sites.

### F4 — Keep `schema.graphql` in lockstep with the API
As filter/limit args are added (F1), update both the API schema and the
frontend's mirrored `schema.graphql` together, per the frontend `CLAUDE.md`.

---

## Suggested order (impact ÷ effort)

1. **B1 indexes** — one migration, no code, immediate broad speedup. ✅
2. **B3 pool size + `.unwrap()` fix** — a few lines, removes a stability cliff. ✅
3. **B2 DataLoaders** on the 5–6 hot edges — the real structural fix for N+1. ✅
4. **B4 depth/complexity limits** — hardening against pathological queries. ✅
5. **F2 / F1 trim + paginate** the list and dropdown queries — frontend-only,
   cuts rows transferred and server fan-out.
6. **F3 round-trip parallelism** — latency polish in the frontend handlers.
