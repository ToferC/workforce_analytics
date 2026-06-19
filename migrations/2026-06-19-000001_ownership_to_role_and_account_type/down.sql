-- Revert ownership from Role back to Person, and drop the user account_type.

ALTER TABLE users DROP COLUMN account_type;

-- team_ownerships: owner_role_id (Role) -> person_id (Person)
ALTER TABLE team_ownerships
    ADD COLUMN person_id UUID REFERENCES persons(id) ON DELETE RESTRICT;

UPDATE team_ownerships t
SET person_id = ra.person_id
FROM role_assignments ra
WHERE ra.role_id = t.owner_role_id
  AND ra.end_date IS NULL;

DELETE FROM team_ownerships WHERE person_id IS NULL;

ALTER TABLE team_ownerships ALTER COLUMN person_id SET NOT NULL;
ALTER TABLE team_ownerships DROP COLUMN owner_role_id;

-- org_tier_ownerships: owner_role_id (Role) -> owner_id (Person)
ALTER TABLE org_tier_ownerships
    ADD COLUMN owner_id UUID REFERENCES persons(id) ON DELETE RESTRICT;

UPDATE org_tier_ownerships o
SET owner_id = ra.person_id
FROM role_assignments ra
WHERE ra.role_id = o.owner_role_id
  AND ra.end_date IS NULL;

DELETE FROM org_tier_ownerships WHERE owner_id IS NULL;

ALTER TABLE org_tier_ownerships ALTER COLUMN owner_id SET NOT NULL;
ALTER TABLE org_tier_ownerships DROP COLUMN owner_role_id;
