-- Tier 1 work tracking: per-item dates (Proposal 1) and actionable BLOCKED
-- context (Proposal 2). All columns are nullable and additive so existing
-- rows and queries are unaffected.
--
-- Proposal 1 — dates on Work:
--   due_date      target completion for this work item (user-set)
--   started_at    stamped server-side when status first moves to IN_PROGRESS
--   completed_at  stamped server-side when status moves to COMPLETED
-- Proposal 2 — actionable BLOCKED:
--   blocked_reason      free text explaining the block (single language,
--                       mirroring work_description)
--   blocked_since       stamped server-side when status moves to BLOCKED,
--                       cleared when it leaves BLOCKED (drives ageing)
--   blocked_on_role_id  the role/position we are waiting on, giving a named,
--                       reachable contact for escalation
ALTER TABLE works
    ADD COLUMN due_date TIMESTAMP,
    ADD COLUMN started_at TIMESTAMP,
    ADD COLUMN completed_at TIMESTAMP,
    ADD COLUMN blocked_reason VARCHAR(1024),
    ADD COLUMN blocked_since TIMESTAMP,
    ADD COLUMN blocked_on_role_id UUID REFERENCES roles(id) ON DELETE SET NULL;
