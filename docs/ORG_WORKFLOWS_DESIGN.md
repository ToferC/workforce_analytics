# Org Workflows Design Dives

Status: **Design only.** Three related next steps along the reporting-spine
progression: (1) seniority validation for `reports_to`, (2) a transfer-as-offer
workflow + manager panel, (3) auditability & reporting. Companion to
`ORG_STRUCTURE_REDESIGN.md`.

These three reinforce each other: the reporting spine routes offer approvals;
offers should be audited from day one; audit + seniority make the hierarchy
trustworthy. Recommended order: **seniority → audit → offers**.

---

## Dive 1 — Seniority validation for `reports_to`

**Goal.** An org is a hierarchy: a role may not report to a peer. "A role cannot
report to another role of the same rank (military or civilian)" — and, more
generally, must report to something *strictly more senior*.

**What we have.** `Role` carries `rank` (military, an ordered enum
Private…General), or `occupational_group` + `occupational_level` (civilian),
even when vacant. `Person.personnel_type` ∈ {Military, Civilian, Contractor,
Student}. Tiers carry `tier_level` (L0 DM/CDS … L4 manager) — a *stream-agnostic*
seniority band. `set_reports_to` already rejects self-reference and cycles.

**The hard part.** Comparing a military rank to a civilian classification is not
well-defined, and civilian levels aren't comparable across groups (EC-07 vs
AS-05). Comparing two enums directly is brittle.

### Options

- **1A — Stream-aware rank/level precedence.** `Role::seniority()` = military
  rank index, or civilian level; compare only within the same stream, skip
  cross-stream. *Brittle: cross-stream undefined, civilian cross-group fuzzy.*
- **1B — Normalized seniority score (recommended).** The genuinely comparable
  axis is the **leadership level** already in the model. Define
  `Role::seniority_score() -> Option<i64>` as a lexicographic
  `(tier_level, within_level_index)` collapsed to one scalar, where
  `within_level_index` is the military rank index or the civilian level. Lower
  `tier_level` dominates (more senior), so cross-stream comparisons resolve on
  tier; same-tier ties break on the stream index. Peers (same score) are
  rejected, which *is* the "same rank can't report to same rank" rule.
- **1C — Stored seniority band enum.** Map every rank/classification to an
  explicit band. Most accurate, most upkeep. Defer.

### Recommendation: 1B + graceful nulls

Rule in `set_reports_to` / `create_role`:

1. Always: no self-reference, no cycle (done).
2. If **both** roles have a known `seniority_score`: require
   `manager.score > role.score` (strictly more senior). Rejects peer-to-peer and
   inverted lines.
3. If either score is **unknown** (vacant role missing rank, contractor/student
   with no rank): **allow but flag a soft warning** rather than hard-block —
   blocking legitimate setups on missing HR data is worse than a warning.
4. Strictness is env-gated (mirror `scoped_authz_enabled()`), so it can be
   enforced/relaxed per environment during rollout.

`seniority_score` is **derived**, not stored (no migration): tier_level from the
role's team's org tier; index from rank/level. Expose `seniority` on the GraphQL
`Role` for the UI to show "reports to a more senior position" inline.

**Effort:** small, self-contained, no schema change. Hardens the spine we just
shipped. Do it first.

---

## Dive 2 — Transfer-as-offer workflow + manager panel

**Goal.** Replace the immediate transfer popup
(`transfer_preview` → `assign_role_post` → `assignPersonToRole`) with an
approval flow:

1. A hiring manager finds the best candidate (reuse existing `fuzzyMatches`) and
   **makes an offer** for a vacant role to a person.
2. The offer routes to the person's **current manager**, who **accepts/declines**.
3. On accept, the **transfer executes** atomically (the existing
   `assign_person` transaction), recording career history as today.

**Pattern.** A state machine + approval queue, mirroring `record_flag`'s
review-queue shape (created/decided timestamps, "pending for me" query).

### Entity: `RoleOffer`

| field | meaning |
|---|---|
| `id` | |
| `role_id` | the offered (vacant) position, on the hiring team |
| `person_id` | the candidate |
| `offered_by_role_id` | the hiring manager's position (initiator) |
| `from_role_id` | the candidate's current role (nullable) — routes approval & detects staleness |
| `status` | `Pending` / `Accepted` / `Declined` / `Withdrawn` / `Expired` / `Completed` |
| `message`, `decision_note` | optional justification / reason |
| `decided_by_role_id`, `decided_at` | who actioned |
| `created_at`, `updated_at`, `expires_at?` | |

### State machine

```
Pending --accept--> Accepted --(assign_person in same txn)--> Completed
Pending --decline--> Declined
Pending --withdraw(offerer)--> Withdrawn
Pending --expire--> Expired
```

Accept runs the transfer **in the same DB transaction** as the status change, so
assignment and offer state can never diverge.

### Authz (reuse scoped authz)

- **Create:** `require_manage_role(role_id)` — you manage the hiring team. Operator+.
- **Accept/Decline:** must manage `from_role_id` (the losing manager), or admin.
- **Withdraw:** the offerer, or admin.

### Routing & consent

- "Current manager" = manager of the candidate's current active role (via
  `reports_to` → falls back to team owner). If the candidate has **no** current
  role, there is nothing to approve → the offer can auto-complete (configurable).
- **Optional person consent (phase 2):** some HR flows require the employee to
  accept too. Shape `status` to allow an inserted `PersonAccepted` step later
  without reshaping the table.

### Concurrency (the bits that make it robust)

- **Competing offers:** on accept, re-check the candidate's current role still
  equals `from_role_id`; if they already moved, the offer is **stale** → refuse
  with a clear message instead of silently re-transferring.
- **Target filled meanwhile:** on accept, re-check `role_id` is still vacant;
  conflict → refuse. (Offers target *vacant* roles; rotating an occupant is a
  separate offer.)
- **Idempotency:** deciding an already-decided offer is a no-op error.

### Manager panel (`/{lang}/manage`)

The home for these workflows, scoped to the manager's team(s) — computable now
from the reporting spine (`manager`/`directReports` + owned teams):

- **Roster:** my reports (reports_to tree) and vacant roles.
- **Offers — incoming** (need my decision) and **outgoing** (track).
- **Pending capability validations** and **record flags** for my people.
- **Quick actions:** post a role, start an offer, vacate.

Notifications reuse the SendGrid `Email` infra + `templates/emails/`, plus an
in-app badge driven by `RoleOffer::get_pending_for_manager(role_ids)` (the
`record_flag::get_unresolved` analogue).

### Options

- **2A — Purpose-built `RoleOffer` (recommended now).** Direct, legible, ships fast.
- **2B — Unified "Approvals/Inbox" framework** that also subsumes record flags
  and validations into one queue + one audit surface. More elegant long-term,
  bigger build. Shape `RoleOffer`'s columns so it can fold into 2B later.

---

## Dive 3 — Auditability & reporting

**What we have.** No general audit log (`access_log.rs` is a dead stub;
`messages.rs` is commented out; the Kafka hooks are abandoned). `RoleAssignment`
is a real, domain-specific history for *assignments only*. Other structural
mutations (reports_to, ownership, create/retire) leave only `updated_at`.

### Options

- **3A — Generic append-only `audit_events` table (recommended core).** One row
  per mutation: `occurred_at`, `actor_user_id` + `actor_role_id` (from the JWT
  principal already in the GraphQL context), `action` (e.g.
  `role.reports_to.set`, `offer.accept`), `entity_type` + `entity_id`,
  `payload` JSONB (before/after diff), `correlation_id` (ties a workflow
  together). Written via one `Audit::record(ctx, …)` helper called from mutation
  resolvers — the single choke point for writes.
- **3B — DB triggers / shadow `*_history` tables.** Can't be bypassed by app
  code, but the DB doesn't know *who*/*why* — loses actor & intent, the most
  valuable columns. Heavier, Postgres-specific.
- **3C — Event sourcing / domain events.** Richest (drives notifications +
  read-models from one source) but the most architecture; the abandoned Kafka
  stubs are a caution. Overkill now.

### Recommendation: 3A + targeted domain events

- One append-only `audit_events` table (3A). Grant no UPDATE/DELETE → integrity
  by construction; hash-chaining for tamper-evidence is a later hardening.
- **Capture the actor from the GraphQL context** — this is what makes 3A beat
  3B: the principal is already resolved there for authz.
- Instrument the high-value mutations first: assignment/transfer, reports_to,
  ownership, offer decisions, role/team create/retire.
- Don't duplicate `RoleAssignment`: keep it as the system of record for tenure
  and emit an `audit_event` that *references* it.
- **Reporting:** queries/views over `audit_events` (by entity / actor / action /
  time), plus a few aggregates ("transfers this quarter by org tier",
  "vacancies opened vs closed"). Surface as an admin **Activity** view and
  per-entity **History** tabs. Every `RoleOffer` transition is an audit_event
  with `correlation_id = offer.id`, giving a complete requested→approved→executed
  transfer trail.

---

## Suggested sequence

1. **Seniority validation (Dive 1)** — small, hardens the spine. No schema change.
2. **Audit log (Dive 3, 3A core)** — foundational; land before offers so the
   workflow is audited from the first transition.
3. **Offer workflow + manager panel (Dive 2)** — the largest piece; consumes
   reports_to (routing), `fuzzyMatches` (discovery), audit (trail), email (notify).

## Open decisions

- **Offer consent:** losing-manager approval only, or also require the candidate
  (person) to accept?
- **Seniority strictness:** hard-block vs warn when scores are unknown or
  cross-stream.
- **Audit approach:** confirm app-level `audit_events` (3A) over DB triggers.
</content>
