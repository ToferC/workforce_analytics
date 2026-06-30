-- Scope org_tier and team name uniqueness to the owning organization.
--
-- The base schema declared org_tiers.name_en/name_fr and teams.name_en/name_fr
-- as globally UNIQUE. That is wrong for a multi-organization model: two
-- different organizations may each legitimately have an "Executive" tier or an
-- "Executive Office" team. The global constraint also breaks
-- Organization::create_with_defaults, which seeds a starter hierarchy with
-- fixed names ("Executive"/"Direction", "Executive Office"/...): the first
-- organization created succeeds, but every subsequent create fails with a
-- unique violation on those name columns.
--
-- Replace the per-column global UNIQUE constraints with composite UNIQUE
-- indexes on (organization_id, name_*), so names only have to be unique within
-- a single organization.

ALTER TABLE org_tiers DROP CONSTRAINT IF EXISTS org_tiers_name_en_key;
ALTER TABLE org_tiers DROP CONSTRAINT IF EXISTS org_tiers_name_fr_key;

ALTER TABLE teams DROP CONSTRAINT IF EXISTS teams_name_en_key;
ALTER TABLE teams DROP CONSTRAINT IF EXISTS teams_name_fr_key;

CREATE UNIQUE INDEX IF NOT EXISTS org_tiers_org_name_en_key
    ON org_tiers (organization_id, name_en);
CREATE UNIQUE INDEX IF NOT EXISTS org_tiers_org_name_fr_key
    ON org_tiers (organization_id, name_fr);

CREATE UNIQUE INDEX IF NOT EXISTS teams_org_name_en_key
    ON teams (organization_id, name_en);
CREATE UNIQUE INDEX IF NOT EXISTS teams_org_name_fr_key
    ON teams (organization_id, name_fr);
