-- This file should undo anything in `up.sql`

ALTER TABLE capabilities DROP COLUMN IF EXISTS validated_at;
ALTER TABLE capabilities DROP COLUMN IF EXISTS validated_by_id;

-- Restore the numeric validation history array used by the averaging model.
ALTER TABLE capabilities ADD COLUMN validation_values BIGINT[] NOT NULL DEFAULT '{}';
