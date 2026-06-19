-- Move OrgTier and Team ownership from Person to Role.
--
-- Ownership represents organizational authority/management responsibility,
-- which belongs to the *position* (Role), not the *individual* (Person). When a
-- person changes jobs the responsibility must stay with the role they vacated.
-- This aligns ownership with Product/Task/Work, which are already Role-based.
--
-- Backfill maps each owning person to the role they currently occupy (their open
-- role assignment). Ownership rows that cannot be mapped (owner has no active
-- role) are dropped; development data is regenerated from the dummy-data scripts.

-- org_tier_ownerships: owner_id (Person) -> owner_role_id (Role)
ALTER TABLE org_tier_ownerships
    ADD COLUMN owner_role_id UUID REFERENCES roles(id) ON DELETE RESTRICT;

UPDATE org_tier_ownerships o
SET owner_role_id = ra.role_id
FROM role_assignments ra
WHERE ra.person_id = o.owner_id
  AND ra.end_date IS NULL;

DELETE FROM org_tier_ownerships WHERE owner_role_id IS NULL;

ALTER TABLE org_tier_ownerships ALTER COLUMN owner_role_id SET NOT NULL;
ALTER TABLE org_tier_ownerships DROP COLUMN owner_id;

-- team_ownerships: person_id (Person) -> owner_role_id (Role)
ALTER TABLE team_ownerships
    ADD COLUMN owner_role_id UUID REFERENCES roles(id) ON DELETE RESTRICT;

UPDATE team_ownerships t
SET owner_role_id = ra.role_id
FROM role_assignments ra
WHERE ra.person_id = t.person_id
  AND ra.end_date IS NULL;

DELETE FROM team_ownerships WHERE owner_role_id IS NULL;

ALTER TABLE team_ownerships ALTER COLUMN owner_role_id SET NOT NULL;
ALTER TABLE team_ownerships DROP COLUMN person_id;

-- Distinguish human users from non-human service principals. An AGENT queries
-- the API on behalf of an application or data service. Like ADMIN users, agents
-- are exempt from the "every user maps to a Person" invariant.
ALTER TABLE users
    ADD COLUMN account_type VARCHAR(16) NOT NULL DEFAULT 'HUMAN';
