-- Indexes to support capability-to-requirement matching queries.
--
-- Without these, every capability lookup does a full sequential scan.
-- The hot path is: for each skill required by a role, find all people who
-- hold that capability at or above a given level, excluding retired rows.

-- Primary lookup: all active capabilities for a given skill, ordered by level.
-- Covers: WHERE skill_id = X AND retired_at IS NULL (ORDER BY validated_level)
-- Used by both binary matching and fuzzy matching.
CREATE INDEX IF NOT EXISTS capabilities_skill_id_active_idx
    ON capabilities (skill_id, validated_level)
    WHERE retired_at IS NULL;

-- Batched fuzzy-match lookup: all active capabilities for a set of skills.
-- The partial index keeps it tight by excluding retired rows from the index.
-- When skill_id IN (...) is issued, Postgres bitmap-ANDs the per-key scans.
CREATE INDEX IF NOT EXISTS capabilities_skill_id_idx
    ON capabilities (skill_id)
    WHERE retired_at IS NULL;

-- Supports per-person capability queries (profile views, career history).
CREATE INDEX IF NOT EXISTS capabilities_person_id_idx
    ON capabilities (person_id);

-- Role requirement lookup: every matching request starts by loading the role's
-- requirements. Without this, it scans the full requirements table.
CREATE INDEX IF NOT EXISTS requirements_role_id_idx
    ON requirements (role_id);

-- Skill requirements lookup: used when finding all roles that need a given skill.
CREATE INDEX IF NOT EXISTS requirements_skill_id_idx
    ON requirements (skill_id);
