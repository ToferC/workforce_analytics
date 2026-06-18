-- Work now requires a specific Skill in addition to its CapabilityLevel.
-- Backfill any existing rows that have no skill before enforcing NOT NULL.

-- Prefer a skill in the same domain as the work item.
UPDATE works w
SET skill_id = (
    SELECT s.id FROM skills s
    WHERE s.domain = w.domain
    ORDER BY s.created_at
    LIMIT 1
)
WHERE w.skill_id IS NULL;

-- Fallback for any work whose domain has no skills: attach any skill.
UPDATE works w
SET skill_id = (SELECT id FROM skills ORDER BY created_at LIMIT 1)
WHERE w.skill_id IS NULL;

ALTER TABLE works ALTER COLUMN skill_id SET NOT NULL;
