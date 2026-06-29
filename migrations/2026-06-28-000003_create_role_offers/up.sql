-- A transfer offer: a hiring manager offers a (vacant) role to a person; the
-- person's current manager accepts or declines; on accept the transfer executes
-- atomically. Replaces the immediate "assign and it's done" path for candidates
-- outside the hiring manager's area.
CREATE TYPE role_offer_status AS ENUM (
    'pending',
    'accepted',
    'declined',
    'withdrawn',
    'expired',
    'completed'
);

CREATE TABLE IF NOT EXISTS role_offers (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,

    -- The offered (vacant) position, on the hiring team.
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,

    -- The candidate.
    person_id UUID NOT NULL REFERENCES persons(id) ON DELETE CASCADE,

    -- The hiring manager's position (initiator).
    offered_by_role_id UUID NOT NULL REFERENCES roles(id) ON DELETE RESTRICT,

    -- The candidate's current role at the time of the offer. Lets accept detect
    -- a stale offer (candidate already moved).
    from_role_id UUID REFERENCES roles(id) ON DELETE SET NULL,

    -- The position that must approve this offer: the manager of the candidate's
    -- current role (explicit reports_to, else the team owner). Drives a manager's
    -- "incoming offers" queue. NULL means admin-only approval (no resolvable
    -- manager).
    approver_role_id UUID REFERENCES roles(id) ON DELETE SET NULL,

    status role_offer_status NOT NULL DEFAULT 'pending',

    message TEXT,          -- optional justification from the offerer
    decision_note TEXT,    -- optional reason from the decider

    decided_by_role_id UUID REFERENCES roles(id) ON DELETE SET NULL,
    decided_at TIMESTAMP,
    expires_at TIMESTAMP,

    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS role_offers_status_idx ON role_offers (status);
CREATE INDEX IF NOT EXISTS role_offers_from_role_idx ON role_offers (from_role_id);
CREATE INDEX IF NOT EXISTS role_offers_offered_by_idx ON role_offers (offered_by_role_id);
CREATE INDEX IF NOT EXISTS role_offers_person_idx ON role_offers (person_id);
CREATE INDEX IF NOT EXISTS role_offers_approver_idx ON role_offers (approver_role_id);

-- At most one live (pending) offer of a given role to a given person.
CREATE UNIQUE INDEX IF NOT EXISTS role_offers_one_pending
    ON role_offers (role_id, person_id) WHERE status = 'pending';
