-- Record flags: lightweight self-service corrections. A Person (once they can
-- sign in) can flag an issue with their own record for operators/admins to
-- review and resolve.

CREATE TABLE record_flags (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    person_id UUID NOT NULL,
    message VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    resolved_at TIMESTAMP,
    CONSTRAINT record_flags_person_id_fkey
        FOREIGN KEY (person_id) REFERENCES persons(id) ON DELETE CASCADE
);

CREATE INDEX record_flags_person_id_idx ON record_flags (person_id);
