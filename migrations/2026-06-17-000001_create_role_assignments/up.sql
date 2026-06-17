-- A Role is a durable organizational position. A role_assignment records one
-- person's tenure in that position: start_date when they were assigned, end_date
-- when they left (NULL while they are the current occupant). The closed rows for
-- a person are their career progression through roles.
CREATE TABLE IF NOT EXISTS role_assignments (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,

    role_id UUID NOT NULL,
    FOREIGN KEY(role_id)
        REFERENCES roles(id) ON DELETE CASCADE,

    person_id UUID NOT NULL,
    FOREIGN KEY(person_id)
        REFERENCES persons(id) ON DELETE RESTRICT,

    start_date TIMESTAMP NOT NULL DEFAULT NOW(),
    end_date TIMESTAMP,

    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- A position can have at most one current (open) occupant.
CREATE UNIQUE INDEX IF NOT EXISTS role_assignments_one_open_per_role
    ON role_assignments (role_id) WHERE end_date IS NULL;

CREATE INDEX IF NOT EXISTS role_assignments_role_id_idx ON role_assignments (role_id);
CREATE INDEX IF NOT EXISTS role_assignments_person_id_idx ON role_assignments (person_id);

-- Backfill: every role that currently records an incumbent becomes one tenure.
-- Active roles keep an open tenure (end_date NULL); inactive roles get a closed
-- tenure dated by the role's end_date (falling back to updated_at).
INSERT INTO role_assignments (role_id, person_id, start_date, end_date, created_at, updated_at)
SELECT
    r.id,
    r.person_id,
    r.start_datestamp,
    CASE WHEN r.active THEN NULL ELSE COALESCE(r.end_date, r.updated_at) END,
    r.created_at,
    r.updated_at
FROM roles r
WHERE r.person_id IS NOT NULL;
