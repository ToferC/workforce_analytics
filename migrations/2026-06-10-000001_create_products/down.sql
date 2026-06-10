-- This file should undo anything in `up.sql`

-- Work without a role cannot be represented in the prior schema
DELETE FROM works WHERE role_id IS NULL;

ALTER TABLE works ALTER COLUMN role_id SET NOT NULL;

ALTER TABLE tasks DROP COLUMN IF EXISTS product_id;

DROP TABLE IF EXISTS products;
