ALTER TABLE tasks
    DROP COLUMN approval_status,
    DROP COLUMN approved_by_user_id,
    DROP COLUMN approved_at,
    DROP COLUMN rejection_reason;
DROP TYPE IF EXISTS approval_status;
