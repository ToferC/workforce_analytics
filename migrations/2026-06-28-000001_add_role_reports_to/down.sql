DROP INDEX IF EXISTS roles_reports_to_idx;
ALTER TABLE roles DROP COLUMN IF EXISTS reports_to;
