-- Products group Tasks for organization and delivery in product management
-- cycles. Tasks flow under a product, and multiple people do Work as part
-- of a task that contributes to the product. Work carries its capability
-- requirement (domain and capability_level) so people with the required
-- capabilities can be identified and matched to the work.

CREATE TABLE IF NOT EXISTS products (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,

    organization_id UUID NOT NULL,
    FOREIGN KEY(organization_id)
        REFERENCES organizations(id) ON DELETE RESTRICT,

    product_owner_role_id UUID NOT NULL,
    FOREIGN KEY(product_owner_role_id)
        REFERENCES roles(id) ON DELETE RESTRICT,

    name_en VARCHAR(256) NOT NULL,
    name_fr VARCHAR(256) NOT NULL,
    description_en TEXT NOT NULL,
    description_fr TEXT NOT NULL,

    primary_domain skill_domain NOT NULL,
    url VARCHAR(256),

    product_status work_status NOT NULL DEFAULT 'planning',
    priority priority NOT NULL DEFAULT 'medium',

    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    retired_at TIMESTAMP
);

-- Tasks may flow under a product
ALTER TABLE tasks ADD COLUMN product_id UUID
    REFERENCES products(id) ON DELETE RESTRICT;

-- Work may now be planned with its capability requirement before a
-- person (role) is matched to it, so role_id becomes optional.
ALTER TABLE works ALTER COLUMN role_id DROP NOT NULL;

-- Work may optionally target a specific skill for precise capability
-- matching; otherwise matching falls back to the work's domain.
ALTER TABLE works ADD COLUMN skill_id UUID
    REFERENCES skills(id) ON DELETE RESTRICT;
