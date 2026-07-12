-- Budget allocations: a dollar envelope granted to an org tier for one
-- fiscal year. Set at L1, then distributed ("rolled down") to L2 and L3
-- children as their own allocation rows. One row per tier per fiscal year;
-- updating an allocation replaces the amount in place.
CREATE TABLE IF NOT EXISTS budget_allocations (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    org_tier_id UUID NOT NULL REFERENCES org_tiers(id) ON DELETE CASCADE,

    -- Starting year of the fiscal year (2026 = FY 2026-27).
    fiscal_year INT NOT NULL,
    amount_cents BIGINT NOT NULL CHECK (amount_cents >= 0),

    created_at TIMESTAMP NOT NULL DEFAULT now(),
    updated_at TIMESTAMP NOT NULL DEFAULT now(),

    CONSTRAINT budget_allocation_uniq UNIQUE (org_tier_id, fiscal_year)
);

CREATE INDEX IF NOT EXISTS budget_allocations_tier_idx ON budget_allocations (org_tier_id);
