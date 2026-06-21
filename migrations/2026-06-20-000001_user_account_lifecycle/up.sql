-- User account lifecycle: separate record creation from system access.
--
-- status:  PROVISIONED (created by an operator, cannot log in)
--          -> INVITED   (activation token issued)
--          -> ACTIVE    (password set, can log in)
--          ; DISABLED   (access revoked, record retained)
--
-- An activation token (separate from access_key, which is reserved for the
-- agent/API-key concept) is issued by inviteUser and redeemed by activateAccount.

ALTER TABLE users ADD COLUMN status VARCHAR(16) NOT NULL DEFAULT 'PROVISIONED';

-- Existing users have real passwords and must remain able to sign in.
UPDATE users SET status = 'ACTIVE';

ALTER TABLE users ADD COLUMN activation_token VARCHAR(128);
ALTER TABLE users ADD COLUMN activation_expires_at TIMESTAMP;
CREATE UNIQUE INDEX users_activation_token_key
    ON users (activation_token) WHERE activation_token IS NOT NULL;

-- Enforce the User<->Person invariant with a foreign key. First provision a
-- User for every Person whose user_id does not resolve to a real user (dev data
-- historically inserted random user_ids). Backfilled accounts are PROVISIONED
-- with a unique placeholder email; real emails are set when dummy data is
-- regenerated.
INSERT INTO users (id, hash, email, role, name, access_level, created_at, updated_at, access_key, approved_by_user_uid, account_type, status)
SELECT p.user_id,
       '',
       'provisioned-' || p.user_id::text || '@local.invalid',
       'USER',
       trim(both ' ' from (p.given_name || ' ' || p.family_name)),
       'detailed', now(), now(), '', NULL, 'HUMAN', 'PROVISIONED'
FROM persons p
LEFT JOIN users u ON u.id = p.user_id
WHERE u.id IS NULL;

ALTER TABLE persons
    ADD CONSTRAINT persons_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE RESTRICT;
