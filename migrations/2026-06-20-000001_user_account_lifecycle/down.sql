-- Reverse the account-lifecycle changes. Backfilled provisioned users are left
-- in place (harmless; removing them could orphan persons).

ALTER TABLE persons DROP CONSTRAINT IF EXISTS persons_user_id_fkey;

DROP INDEX IF EXISTS users_activation_token_key;
ALTER TABLE users DROP COLUMN IF EXISTS activation_expires_at;
ALTER TABLE users DROP COLUMN IF EXISTS activation_token;
ALTER TABLE users DROP COLUMN IF EXISTS status;
