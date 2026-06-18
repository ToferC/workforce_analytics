-- Fix validator foreign keys to reference users instead of persons.
-- Validations are performed by admin Users, not by Persons.

-- validations.validator_id: persons → users
ALTER TABLE validations
    DROP CONSTRAINT IF EXISTS validations_validator_id_fkey;

ALTER TABLE validations
    ADD CONSTRAINT validations_validator_id_fkey
    FOREIGN KEY (validator_id) REFERENCES users(id) ON DELETE RESTRICT;

-- capabilities.validated_by_id: persons → users
ALTER TABLE capabilities
    DROP CONSTRAINT IF EXISTS capabilities_validated_by_id_fkey;

ALTER TABLE capabilities
    ADD CONSTRAINT capabilities_validated_by_id_fkey
    FOREIGN KEY (validated_by_id) REFERENCES users(id) ON DELETE RESTRICT;
