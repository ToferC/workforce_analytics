-- Tier 2, Proposal 3 — comment / "needs attention" flag stream on Work.
--
-- Gives the two personas a two-way channel that lives with the work item:
-- an assignee can leave a comment or raise a FLAG for management attention,
-- and a manager can resolve flags. Attribution (author, resolver) comes from
-- the request JWT. Kind is COMMENT or FLAG; only FLAG rows use the resolve
-- columns.
CREATE TYPE work_update_kind AS ENUM ('comment', 'flag');

CREATE TABLE IF NOT EXISTS work_updates (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,

    work_id UUID NOT NULL REFERENCES works(id) ON DELETE CASCADE,
    author_user_id UUID REFERENCES users(id) ON DELETE SET NULL,

    kind work_update_kind NOT NULL DEFAULT 'comment',
    body TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT now(),

    -- Set when a FLAG is cleared; NULL means an open flag (or a plain comment).
    flag_resolved_at TIMESTAMP,
    resolved_by_user_id UUID REFERENCES users(id) ON DELETE SET NULL
);

CREATE INDEX idx_work_updates_work_id ON work_updates (work_id, created_at DESC);
