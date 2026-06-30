-- Revert to globally UNIQUE org_tier and team names.
--
-- Note: re-adding the global UNIQUE constraints will fail if duplicate names
-- exist across organizations (which the up migration intentionally permits).

DROP INDEX IF EXISTS org_tiers_org_name_en_key;
DROP INDEX IF EXISTS org_tiers_org_name_fr_key;
DROP INDEX IF EXISTS teams_org_name_en_key;
DROP INDEX IF EXISTS teams_org_name_fr_key;

ALTER TABLE org_tiers ADD CONSTRAINT org_tiers_name_en_key UNIQUE (name_en);
ALTER TABLE org_tiers ADD CONSTRAINT org_tiers_name_fr_key UNIQUE (name_fr);

ALTER TABLE teams ADD CONSTRAINT teams_name_en_key UNIQUE (name_en);
ALTER TABLE teams ADD CONSTRAINT teams_name_fr_key UNIQUE (name_fr);
