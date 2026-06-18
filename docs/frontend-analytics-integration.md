# Updated Analytics API — Frontend Integration Guide

## What Changed

The analytics have been split into **four independent GraphQL queries** (instead of one monolithic block), and **lightweight analytics are now available directly on `OrgTier` and `Team` objects**.

---

## 1. Inline Analytics on OrgTier and Team

These are the lightest-weight option — include them alongside existing queries without a separate request.

### On OrgTier

```graphql
query {
  allOrgTiers {
    id
    nameEn
    tierLevel
    headcount                    # number of people under this tier
    capabilityHeatmap {          # per-team capability breakdown
      teamId
      teamName
      cells {
        domain
        depth                   # weighted capability score
        peopleCount             # people contributing to this domain
      }
    }
    # ...existing fields
  }
}
```

### On Team

```graphql
query {
  allTeams {
    id
    nameEnglish
    headcount                    # distinct active people on this team
    capabilityHeatmap {          # domain-level capability summary
      domain
      depth
      peopleCount
    }
    capabilityCounts { ... }     # already existed
    totalEffort                  # already existed
  }
}
```

These are cheap enough to include in org chart cards — e.g. showing headcount badges or a small domain-strength indicator per node.

---

## 2. Dedicated Analytics Queries (for analytics pages)

Each can be called independently so you only pay for what you need.

### A. Capability Growth (time series)

```graphql
query CapGrowth($bucket: TimeBucket!, $from: NaiveDateTime, $to: NaiveDateTime, $orgTierId: ID) {
  capabilityGrowth(bucket: $bucket, from: $from, to: $to, orgTierId: $orgTierId) {
    key              # SkillDomain name (e.g. "SoftwareEngineering")
    points {
      periodStart
      bucket
      value          # cumulative validated capability depth
    }
  }
}
```

### B. Supply vs Demand

```graphql
query SupplyDemand($bucket: TimeBucket!, $orgTierId: ID) {
  capabilitySupplyDemand(bucket: $bucket, orgTierId: $orgTierId) {
    domain
    points {
      periodStart
      bucket
      supply
      demand
    }
  }
}
```

### C. Team Capability Matrix (heatmap)

```graphql
query TeamMatrix($orgTierId: ID) {
  teamCapabilityMatrix(orgTierId: $orgTierId) {
    teamId
    teamName
    cells {
      domain
      depth
      peopleCount
    }
  }
}
```

### D. Talent Movements

```graphql
query Movements($from: NaiveDateTime, $to: NaiveDateTime, $orgTierId: ID) {
  talentMovements(from: $from, to: $to, orgTierId: $orgTierId) {
    personId
    at
    fromTeamId
    toTeamId
    fromLevel
    toLevel
    kind             # "PROMOTION" | "LATERAL" | "INFLOW" | "OUTFLOW"
  }
}
```

---

## 3. Suggested Page Structure

| Page | Query | Use case |
|---|---|---|
| Org Chart | `allOrgTiers` with inline `headcount` | Show people count on each node |
| Team Detail | `Team.capabilityHeatmap` + `Team.headcount` | Domain strength at a glance |
| Capability Growth | `capabilityGrowth` | Line chart of skill accumulation |
| Supply/Demand | `capabilitySupplyDemand` | Gap analysis chart |
| Capability Heatmap | `teamCapabilityMatrix` | Cross-team domain comparison grid |
| Talent Mobility | `talentMovements` | Sankey/flow diagram of promotions & moves |

---

## 4. Scoping with `orgTierId`

All analytics accept an optional `orgTierId` parameter. When provided, results are scoped to that tier and all descendants. This lets you build drill-down views — click an org chart node to see analytics for just that subtree.

---

## 5. `TimeBucket` Enum Values

For time-series queries, `bucket` accepts: `WEEK`, `MONTH`, `QUARTER`, `YEAR`.

---

## 6. `SkillDomain` Enum Values

The `key` / `domain` fields in analytics responses use these domain names:

- **Military**: `Combat`, `Intelligence`, `Strategy`, `Engineering`, `Medical`, `JointOperations`
- **Technology**: `SoftwareEngineering`, `CloudPlatformDevOps`, `DataAnalyticsAndAi`, `CyberSecurity`
- **Delivery**: `ProductAgileAndDelivery`, `UserExperience`
- **Organizational**: `ProcurementAndVendorManagement`, `PeopleAndOrganisationalLeadership`, `Governance`, `CorporateServices`

---

## 7. Existing Frontend Context

The React app lives in `/orgchart` and uses Apollo Client (`@apollo/client`) with `useQuery` and `gql`. The current `GET_PEOPLE` query in `App.tsx` fetches `allOrgTiers` — the new `headcount` and `capabilityHeatmap` fields can be added directly to that query.

The app uses Material UI (`@mui/material`) for components and `react-organizational-chart` for the tree layout.
