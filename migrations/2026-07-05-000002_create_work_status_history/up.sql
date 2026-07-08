-- Tier 2, Proposal 4 — status history for Work.
--
-- Append-only log of every work_status transition, so the app can report on
-- ageing, cycle time, and "what changed since <date>" instead of only knowing
-- the current point-in-time status. `from_status` is NULL for the row that
-- records a work item's initial status at creation. `changed_by_user_id` is
-- the acting user (from the request JWT); nullable and ON DELETE SET NULL so
-- history survives user deletion.
CREATE TABLE IF NOT EXISTS work_status_history (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,

    work_id UUID NOT NULL REFERENCES works(id) ON DELETE CASCADE,

    from_status work_status,
    to_status work_status NOT NULL,

    changed_at TIMESTAMP NOT NULL DEFAULT now(),
    changed_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL
);

-- History is always read by work, most-recent first.
CREATE INDEX idx_work_status_history_work_id ON work_status_history (work_id, changed_at DESC);
