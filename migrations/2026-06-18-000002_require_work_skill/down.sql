-- Allow work items without a specific skill again.
ALTER TABLE works ALTER COLUMN skill_id DROP NOT NULL;
