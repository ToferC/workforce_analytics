-- Indexes on the foreign-key columns traversed most heavily by the GraphQL
-- resolvers. Without these, each per-row resolver lookup (Team -> roles,
-- Role -> work, Work -> task, Task -> product, etc.) falls back to a
-- sequential scan, which compounds badly under the resolvers' N+1 access
-- pattern. Plain (non-CONCURRENT) creation so the statements run inside the
-- embedded migration transaction; switch to CREATE INDEX CONCURRENTLY in a
-- standalone migration if these tables grow large enough that build-time
-- locking on a live database becomes a concern.

CREATE INDEX IF NOT EXISTS roles_team_id_idx ON roles (team_id);
CREATE INDEX IF NOT EXISTS roles_person_id_idx ON roles (person_id);

CREATE INDEX IF NOT EXISTS works_role_id_idx ON works (role_id);
CREATE INDEX IF NOT EXISTS works_task_id_idx ON works (task_id);

CREATE INDEX IF NOT EXISTS tasks_product_id_idx ON tasks (product_id);
CREATE INDEX IF NOT EXISTS tasks_created_by_role_id_idx ON tasks (created_by_role_id);

CREATE INDEX IF NOT EXISTS teams_org_tier_id_idx ON teams (org_tier_id);
