-- Add an explicit reporting edge between positions (Roles).
--
-- reports_to points to the Role this position reports to. It is position->
-- position (not person->person): you report to a position, which may be vacant.
-- NULL means "reports to my team's owner role" (resolved at read time), so the
-- column is backwards-compatible and only set when a position needs a manager
-- other than the default team owner (e.g. a team lead reporting to a manager
-- inside the same team, or a cross-team/cross-tier line).
--
-- ON DELETE SET NULL: removing a manager position should leave its reports
-- falling back to the team owner, not cascade-delete the org beneath it.
ALTER TABLE roles
    ADD COLUMN reports_to UUID REFERENCES roles(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS roles_reports_to_idx ON roles (reports_to);

-- Backfill the implied edges that were previously only expressed through
-- ownership, so existing data reads the same after the change.

-- 1. Every non-owner role reports to its team's owner role.
UPDATE roles r
SET reports_to = t.owner_role_id
FROM team_ownerships t
WHERE r.team_id = t.team_id
  AND r.id <> t.owner_role_id;

-- 2. Each team owner role reports up to the role that owns the team's org tier
--    (skip when they are the same role, to avoid a self-reference). The tier
--    tree is a DAG and ownership flows upward, so this cannot create a cycle.
UPDATE roles r
SET reports_to = oto.owner_role_id
FROM team_ownerships t
JOIN teams tm ON tm.id = t.team_id
JOIN org_tier_ownerships oto ON oto.org_tier_id = tm.org_tier_id
WHERE r.id = t.owner_role_id
  AND r.id <> oto.owner_role_id;
