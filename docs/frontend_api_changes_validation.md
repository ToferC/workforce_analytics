# Frontend Update Guide: Central-Authority Validation

> **Audience:** `workforce_frontend` maintainers
> **Source change:** `workforce_analytics` PR #11 — "Replace averaged validations with single central-authority validation"
> **Date:** 2026-06-16

This document describes the GraphQL API changes introduced when capability
validation moved from an **averaging model** (many people's validations
averaged into a level) to a **single central-authority model** (one
authoritative validation sets the level directly). It lists every schema
change that affects the frontend and the updates required to stay compatible.

---

## TL;DR — what the frontend must do

1. **Stop sending `validationValues`** in the `NewCapability` input — the field no longer exists.
2. **Stop computing or displaying an "average" validation** — there is no average anymore. The level shown is the authoritative `validatedLevel`.
3. **Optionally surface provenance** — new `validatedBy`, `validatedById`, and `validatedAt` fields on `Capability` tell you *who* set the current level and *when*.
4. **Adjust validation UX copy** — a validation now *sets* the level directly (latest authoritative validation wins); it is no longer one vote among many.

---

## Conceptual change

| | Before (averaging) | After (central authority) |
|---|---|---|
| How `validatedLevel` is set | Average of all validations, mapped to the nearest `CapabilityLevel` | Set **directly** from the most recent authoritative validation |
| Who validates | Any number of validators contribute | A central authority (Admin-guarded `createValidation` / `updateValidation`) |
| History | Numeric values accumulated in a hidden array | Each validation kept as a date-stamped row (provenance log) |
| Provenance on the capability | None | `validatedBy`, `validatedById`, `validatedAt` |

Validations are **still recorded individually and date-stamped**, so any UI
that lists a capability's validation history (`Capability.validations`)
continues to work unchanged.

---

## Schema changes

### 1. `Capability` type — new provenance fields (additive)

```graphql
type Capability {
  id: UUID!
  nameEn: String!
  nameFr: String!
  domain: SkillDomain!
  skillId: UUID!
  organizationId: UUID!
  selfIdentifiedLevel: CapabilityLevel!
  validatedLevel: CapabilityLevel        # unchanged: nullable until validated
  createdAt: NaiveDateTime!
  updatedAt: NaiveDateTime!
  retiredAt: NaiveDateTime

  # NEW — provenance of the current validated level
  validatedById: UUID                    # the authority's Person id, null if never validated
  validatedAt: NaiveDateTime             # when the level was set, null if never validated

  person: Person!
  validatedBy: Person                    # NEW — the authority Person, null if never validated
  skillName: String!
  skill: Skill!
  "Detailed view of validations for this capability"
  validations: [Validation!]!            # unchanged: full date-stamped history
}
```

**Frontend action:** these are additive and optional. Add them to your
capability fragments/queries if you want to display "Validated by X on
DATE". No breaking change if you ignore them.

### 2. `NewCapability` input — removed field (**breaking**)

```diff
 input NewCapability {
   nameEn: String!
   nameFr: String!
   domain: SkillDomain!
   personId: UUID!
   skillId: UUID!
   organizationId: UUID!
   selfIdentifiedLevel: CapabilityLevel!
-  validationValues: [Int!]!
 }
```

**Frontend action (required):** remove `validationValues` from any
`createCapability` mutation variables/inputs. Sending it will now cause a
GraphQL validation error ("unknown field").

### 3. `Validation` type — unchanged shape

The `Validation` type fields are unchanged (`id`, `validatorId`,
`capabilityId`, `validatedLevel`, `createdAt`, `updatedAt`, `validator`).
Only its description was updated to reflect the central-authority meaning.
**No frontend action needed.**

### 4. Mutations — same signatures, new behavior

```graphql
createValidation(data: NewValidation!): Validation!
updateValidation(data: ValidationData!): Validation!
```

Signatures are unchanged, so existing calls keep compiling. The **behavior**
changed:

- `createValidation` now sets the capability's `validatedLevel` **directly**
  from the submitted `validatedLevel` (no averaging), and stamps
  `validatedById` / `validatedAt` onto the capability.
- `updateValidation` re-applies the authoritative level the same way.
- Both remain **Admin-only** (the central authority).

**Frontend action:** after calling either mutation, re-fetch the affected
`Capability` (or update your cache) — `validatedLevel`, `validatedById`,
and `validatedAt` may all have changed. Remove any client-side averaging
logic; the value returned by the server is authoritative.

---

## Removed concepts

- **`ValidatedLevel` / `average`** — this object (capability level + average)
  was never exposed in the schema, but if any frontend code or generated
  types referenced an "average" validation value, it no longer exists.
- **`validationValues`** — the numeric history array is gone (see input change above).

---

## Example: before → after

**Create a capability (before):**

```graphql
mutation {
  createCapability(data: {
    nameEn: "Rust", nameFr: "Rust",
    domain: SoftwareDevelopment,
    personId: "…", skillId: "…", organizationId: "…",
    selfIdentifiedLevel: Experienced,
    validationValues: [200]          # ← remove this
  }) { id }
}
```

**Create a capability (after):**

```graphql
mutation {
  createCapability(data: {
    nameEn: "Rust", nameFr: "Rust",
    domain: SoftwareDevelopment,
    personId: "…", skillId: "…", organizationId: "…",
    selfIdentifiedLevel: Experienced
  }) { id }
}
```

**Validate and read provenance (after):**

```graphql
mutation {
  createValidation(data: {
    validatorId: "…", capabilityId: "…", validatedLevel: Expert
  }) {
    id
    validatedLevel
    capability {
      validatedLevel    # now == Expert, set directly
      validatedAt
      validatedBy { id name }
    }
  }
}
```

---

## Suggested UI changes

- Replace any "average validation" indicator with a single "Validated level"
  badge sourced from `validatedLevel`.
- Add a "Validated by {name} on {date}" line using `validatedBy` /
  `validatedAt`. Show "Not yet validated" when `validatedLevel` is null.
- Keep the validation history list (`Capability.validations`) as a provenance
  trail; consider labelling the most recent one as "current authoritative
  validation".
- Gate the validation action behind admin/authority permissions (the mutation
  already enforces this server-side).

---

## Compatibility checklist

- [ ] Removed `validationValues` from all create-capability inputs.
- [ ] Removed client-side validation averaging.
- [ ] Updated capability fragments to include `validatedBy` / `validatedAt` (optional).
- [ ] Re-fetch / cache-update capability after validating.
- [ ] Regenerated GraphQL types from the updated `schema.graphqls`.
- [ ] Updated validation UI copy to "sets the level" semantics.
