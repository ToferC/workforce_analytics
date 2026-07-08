-- Tier 3, Proposal 7a — work dependencies.
--
-- Makes "blocked by" a real relationship instead of only a status: a work item
-- may depend on other work items that must finish first. `work_id` is blocked
-- by `depends_on_work_id`. Self-dependency is rejected at the table; cycles are
-- prevented in the mutation before insert.
CREATE TABLE IF NOT EXISTS work_dependencies (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    work_id UUID NOT NULL REFERENCES works(id) ON DELETE CASCADE,
    depends_on_work_id UUID NOT NULL REFERENCES works(id) ON DELETE CASCADE,
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    UNIQUE (work_id, depends_on_work_id),
    CHECK (work_id <> depends_on_work_id)
);

CREATE INDEX idx_work_deps_work ON work_dependencies (work_id);
CREATE INDEX idx_work_deps_dep ON work_dependencies (depends_on_work_id);
