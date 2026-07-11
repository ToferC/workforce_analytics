-- Financial module.
--
-- 1. pay_rates: prices a classification (civilian occupational group + level,
--    or military rank) with an annual salary. A role's salary is derived from
--    its classification via the most recent rate whose effective_date has
--    passed, so rates are maintained in one place and vacant roles are still
--    budgetable. Superseding a rate = inserting a newer effective_date row.
CREATE TABLE IF NOT EXISTS pay_rates (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,

    -- Exactly one classification: civilian (group + level) or military (rank),
    -- mirroring the same duality on roles.
    occupational_group occupational_group,
    occupational_level INT,
    rank rank,

    annual_rate_cents BIGINT NOT NULL CHECK (annual_rate_cents >= 0),
    effective_date TIMESTAMP NOT NULL,

    created_at TIMESTAMP NOT NULL DEFAULT now(),
    updated_at TIMESTAMP NOT NULL DEFAULT now(),

    CONSTRAINT pay_rate_classification CHECK (
        (rank IS NOT NULL AND occupational_group IS NULL AND occupational_level IS NULL)
        OR (rank IS NULL AND occupational_group IS NOT NULL AND occupational_level IS NOT NULL)
    )
);

CREATE UNIQUE INDEX IF NOT EXISTS pay_rates_civilian_uniq
    ON pay_rates (occupational_group, occupational_level, effective_date)
    WHERE rank IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS pay_rates_military_uniq
    ON pay_rates (rank, effective_date)
    WHERE rank IS NOT NULL;

-- 2. Per-role salary override for positions priced off-scale. NULL means
--    "use the pay_rates lookup".
ALTER TABLE roles ADD COLUMN IF NOT EXISTS annual_salary_cents BIGINT;

-- 3. contracts: procurement spend recorded under a task. Value is recognized
--    linearly across [start_date, end_date]; a fiscal year's share is its
--    day-overlap with that period. Amendments are additional rows.
CREATE TYPE contract_status AS ENUM ('planned', 'active', 'closed');

CREATE TABLE IF NOT EXISTS contracts (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,

    reference_number VARCHAR(64) NOT NULL,
    vendor VARCHAR(256) NOT NULL,
    description TEXT NOT NULL DEFAULT '',

    start_date TIMESTAMP NOT NULL,
    end_date TIMESTAMP NOT NULL,
    total_value_cents BIGINT NOT NULL CHECK (total_value_cents >= 0),
    status contract_status NOT NULL DEFAULT 'active',

    created_at TIMESTAMP NOT NULL DEFAULT now(),
    updated_at TIMESTAMP NOT NULL DEFAULT now(),

    CONSTRAINT contract_period CHECK (end_date >= start_date)
);

CREATE INDEX IF NOT EXISTS contracts_task_id_idx ON contracts (task_id);
