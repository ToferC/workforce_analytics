-- Shift validation from averaging many people's assessments to a single
-- authoritative validation that sets the validated level directly. The
-- validations table is retained as the date-stamped provenance log, while
-- the capability records which authority set the current level and when.

-- The averaged numeric history is no longer used to compute the level.
ALTER TABLE capabilities DROP COLUMN IF EXISTS validation_values;

-- Provenance of the current validated level: the validating authority and
-- the moment they set it. Reconstructable from the validations log, but
-- denormalized here for cheap querying.
ALTER TABLE capabilities ADD COLUMN validated_by_id UUID
    REFERENCES persons(id) ON DELETE RESTRICT;

ALTER TABLE capabilities ADD COLUMN validated_at TIMESTAMP;
