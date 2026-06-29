-- Append-only audit trail of changes made through the API.
--
-- One row per mutating action. The actor is captured from the authenticated
-- GraphQL principal (the DB itself doesn't know who/why, which is exactly why
-- this lives at the app layer rather than in triggers). There are deliberately
-- NO foreign keys to actor/entity: the audit trail must survive deletion of the
-- things it describes, and must never block a delete.
--
-- Treat as write-once: grant no UPDATE/DELETE in production. Tamper-evidence
-- (hash chaining) is a possible later hardening.
CREATE TABLE IF NOT EXISTS audit_events (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    occurred_at TIMESTAMP NOT NULL DEFAULT NOW(),

    -- Who acted. Both nullable: system/agent actions and unauthenticated paths
    -- may have neither a user nor a resolvable role.
    actor_user_id UUID,
    actor_role_id UUID,

    -- What happened: a dotted action key (e.g. "role.reports_to.set") and the
    -- primary object it concerns.
    action VARCHAR(64) NOT NULL,
    entity_type VARCHAR(48) NOT NULL,
    entity_id UUID,

    -- Human-readable one-liner and a structured before/after (or relevant
    -- fields) payload for detail views and reporting.
    summary TEXT,
    payload JSONB,

    -- Ties together the steps of a single workflow (e.g. every transition of a
    -- RoleOffer shares the offer's id).
    correlation_id UUID
);

CREATE INDEX IF NOT EXISTS audit_events_entity_idx ON audit_events (entity_type, entity_id);
CREATE INDEX IF NOT EXISTS audit_events_occurred_at_idx ON audit_events (occurred_at DESC);
CREATE INDEX IF NOT EXISTS audit_events_actor_idx ON audit_events (actor_user_id);
CREATE INDEX IF NOT EXISTS audit_events_correlation_idx ON audit_events (correlation_id);
