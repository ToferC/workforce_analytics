# Design: User/Person Onboarding & Activation Lifecycle

**Status:** Proposed
**Scope:** `workforce_analytics` (API) and `workforce-frontend` (UI)
**Supersedes parts of:** the prior "Person-as-User & Manager-Scoped Authorization" roadmap (PRs #17/#18/#35)

## 1. Why this doc

The prior roadmap bundled two concerns. One — **manager-scoped authorization** — is already shipped:
`graphql/authz.rs` (`effective_scope`, `can_manage_*`) gates OrgTier/Team/Role/Task/Work/Product
mutations to a principal's owned subtree (PR #20). Its Phases 0/3/4 are therefore **done** and are
not repeated here.

The remaining gap is the **User ↔ Person lifecycle**, and the requirement has changed since that
roadmap was written:

> A responsible operator or admin must be able to create a **User and Person without that person
> being able to access the system**. The person then has a **path to later gain access** to update
> their own information, flag errors, etc.

The prior plan auto-issued an invite token at creation. That conflates *record creation* with
*granting access*. This design **separates them** into an explicit account lifecycle.

## 2. The model in one picture

```
                       createPerson                inviteUser / grantAccess        activateAccount
 (operator/admin)  ───────────────────▶  PROVISIONED ───────────────────▶ INVITED ───────────────▶ ACTIVE
   no login possible        │  User+Person created atomically   │  activation token issued   │  password set,
   record fully usable      │  no password, cannot sign in      │  (link surfaced/emailed)   │  can sign in
                            │                                   │                            │
                            └────────── disableUser (admin) ◀───┴──────── disableUser ◀──────┘
                                              ▼
                                          DISABLED  (no login; data retained; re-enablable)
```

A **PROVISIONED** account is pure data: the Person can be placed in roles, owned tiers, matched, etc.,
but **cannot log in**. Access is granted only by the deliberate `inviteUser` → `activateAccount`
path. This is the core of the requirement.

## 3. Account state

Add a `status` to `User` (orthogonal to the existing `account_type` HUMAN/AGENT from PR #20):

| Status | Can sign in? | Meaning |
|---|---|---|
| `PROVISIONED` | No | Created by an operator/admin. No usable password. **Default for created humans.** |
| `INVITED` | No | An activation token has been issued (link surfaced/emailed). Awaiting redemption. |
| `ACTIVE` | Yes | Password set by the person (or the bootstrap admin). |
| `DISABLED` | No | Access revoked; record retained; re-enablable. |

- The **bootstrap admin** and existing seeded/admin users are `ACTIVE`.
- **Agents** (`account_type = AGENT`) are created `ACTIVE` (they authenticate by token, not this flow).
- `sign_in` succeeds only when `status = ACTIVE` **and** the password verifies.

## 4. Data model changes

- `users.status` — new enum column, `NOT NULL DEFAULT 'PROVISIONED'`.
- `users.activation_token` — `Nullable<Varchar>`, unique when present. Distinct from `access_key`
  (which stays reserved for the agent/API-key concept) to avoid overloading one column with two
  meanings.
- `users.activation_expires_at` — `Nullable<Timestamp>`.
- **`persons.user_id → users.id` foreign key** (column is already `UNIQUE NOT NULL`). This finally
  enforces the User↔Person invariant at the DB.

**Migration / backfill:** create one `PROVISIONED` user per Person whose `user_id` doesn't resolve
to a real user (using the Person's email/name; dedup against `users.email`), then add the FK. Mark
pre-existing real users/admins `ACTIVE`. Reversible down migration.

> Note: this also fixes a long-standing bug — `dummy_org_data.rs` currently inserts Persons with
> **random `user_id`s** that match no user. The FK forces us to seed real users (see §7).

## 5. API surface

**Creation (decoupled from access):**
- Replace caller-supplied `user_id` on person creation. `createPerson` (operator+) becomes a single
  **transaction**: create `User { status: PROVISIONED, account_type: HUMAN, role: USER, no password }`
  from the person's email + name, then create the linked `Person`. `NewPerson` loses `user_id` and
  gains nothing the operator must know about accounts. Email is required (it is the future login id
  and invite target); nothing is sent.

**Granting access (deliberate, later):**
- `inviteUser(userId)` (operator/admin) → generates `activation_token` + expiry, sets `status =
  INVITED`, returns the activation link. Email delivery is optional/out-of-scope for v1 — the link is
  surfaced to the operator to share. Requires the user to have an email.
- `activateAccount(token, password)` (**public**) → validates the token + expiry, sets the password
  (Argon2), `status = ACTIVE`, clears the token.
- `disableUser(userId)` / `enableUser(userId)` (admin) → `DISABLED` ⇄ prior state.

**Identity & self-service (the "update info, flag errors" path):**
- `me` → the authenticated caller's `User` + linked `Person` (the join the UI needs once someone logs
  in). Reuses the user id already in the request context.
- `updateMyPerson(input)` → a Person may edit **their own** contact fields (and self-identified
  capability levels) — authorized by `person.user_id == caller`, independent of the operator-scoped
  guards. Role stays `USER`, so they still cannot touch org structure.
- `flagRecordIssue(message)` → lightweight correction request attached to the caller's Person,
  visible to operators/admins. (MVP: a single `record_flags` row {person_id, message, created_at,
  resolved_at}. Avoids a heavier messaging build.)

Everything else (manager-scoped mutation guards) is unchanged — already shipped.

## 6. Frontend (workforce-frontend)

- **Person create form:** drop the "user account email" lookup (`get_user_by_email`) and the
  user-must-exist-first error path. Creating a Person now provisions its account automatically.
- **Person view / admin:** show an account-status chip (Provisioned / Invited / Active / Disabled) and
  a **"Grant access"** action (operator/admin) that calls `inviteUser` and displays the activation
  link to copy/send. Plus enable/disable for admins.
- **Activation page (public):** `/{lang}/activate?token=…` → set-password form → `activateAccount` →
  redirect to login. Works as plain POST + redirect (HTMX optional).
- **Self-service "My profile":** once logged in, a page (driven by `me`) where the person edits their
  own contact info and submits `flagRecordIssue`. Surface it in the nav for authenticated users.
- **Builder integration:** "add a person to this role" opens the create-person flow (provisioned) and
  assigns in one pass, instead of bouncing to `/person/new`.
- Re-sync `schema.graphql` with the API; gate the new buttons with `require_role` (server remains
  authoritative).

## 7. Dummy data

Seed a real `User` for every generated Person (status `ACTIVE` for a handful of demo logins,
`PROVISIONED` for the rest so the provisioned→invite→activate flow is demonstrable). Keep the
bootstrap admin `ACTIVE`. Remove the random-`user_id` hack so the new FK holds.

## 8. What we deliberately drop from the prior roadmap

| Prior roadmap item | Disposition |
|---|---|
| Phase 0 — `managed_tier_ids`, ownership predicates | **Done** (`graphql/authz.rs`) |
| Phase 3 — `OwnershipGuard` on OrgTier/Team/role | **Done** (scoped guards, PR #20) |
| Phase 4 — Products/Tasks/Work scoped to managers | **Done** (PR #20) |
| Auto-issue invite token at Person creation | **Rejected** — creation must not grant access; invite is a separate step |
| Per-request `managed_tier_ids` cache (2.2) | Optional perf follow-up; not required |
| React `/orgchart` as the frontend | Superseded — target is `workforce-frontend` |
| Explicit `manager_person_id` on products | Moot — product owner is a Role, already scoped |

## 9. Phasing

1. **API lifecycle core** — migration (`status`, token cols, FK) + backfill; `User` status/sign-in
   gating; transactional `createPerson`; `inviteUser` / `activateAccount` / `disable`/`enable`;
   dummy-data fix. Ships the "create without access + activate later" capability.
2. **API self-service** — `me`, `updateMyPerson`, `flagRecordIssue`.
3. **Frontend** — provisioned create form, status chip + Grant access, activation page, My-profile +
   flagging, builder hook; schema re-sync.

API leads each phase; the frontend follows once `schema.graphqls` is regenerated.

## 10. Resolved decisions

- **Email at creation:** **required** — it is the login id and invite target; `users.email` stays
  `NOT NULL UNIQUE`.
- **Activation delivery:** v1 **surfaces the activation link to the operator** (`inviteUser` returns
  it). No SendGrid dependency yet; email delivery is a later enhancement.
- **Flagging:** a **minimal `record_flags`** row `{id, person_id, message, created_at, resolved_at}`
  the person submits and operators resolve — not the richer `messages` model.
