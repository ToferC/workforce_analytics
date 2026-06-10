-- This file should undo anything in `up.sql`

ALTER TABLE works DROP COLUMN IF EXISTS product_id;

-- Work without a task or role cannot be represented in the prior schema
DELETE FROM works WHERE task_id IS NULL OR role_id IS NULL;

ALTER TABLE works ALTER COLUMN task_id SET NOT NULL;
ALTER TABLE works ALTER COLUMN role_id SET NOT NULL;

DROP TABLE IF EXISTS products;
