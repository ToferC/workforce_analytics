-- Tier 3, Proposal 7b — approval state for Task.
--
-- Replaces the opaque `approval_tier` integer (kept as the *level required*)
-- with a real workflow *state*: a task moves DRAFT -> PENDING_APPROVAL ->
-- APPROVED | REJECTED, and records who approved/rejected, when, and why.
CREATE TYPE approval_status AS ENUM ('draft', 'pending_approval', 'approved', 'rejected');

ALTER TABLE tasks
    ADD COLUMN approval_status approval_status NOT NULL DEFAULT 'draft',
    ADD COLUMN approved_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    ADD COLUMN approved_at TIMESTAMP,
    ADD COLUMN rejection_reason TEXT;
