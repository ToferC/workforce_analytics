-- Revert validator foreign keys back to persons.

ALTER TABLE validations
    DROP CONSTRAINT IF EXISTS validations_validator_id_fkey;

ALTER TABLE validations
    ADD CONSTRAINT validations_validator_id_fkey
    FOREIGN KEY (validator_id) REFERENCES persons(id) ON DELETE RESTRICT;

ALTER TABLE capabilities
    DROP CONSTRAINT IF EXISTS capabilities_validated_by_id_fkey;

ALTER TABLE capabilities
    ADD CONSTRAINT capabilities_validated_by_id_fkey
    FOREIGN KEY (validated_by_id) REFERENCES persons(id) ON DELETE RESTRICT;
