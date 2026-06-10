-- Products group Work elements for organization and delivery in product
-- management cycles. Work is planned under a product with its capability
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

    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    retired_at TIMESTAMP
);

-- Work may now be planned under a product before a task or role is known,
-- so task_id and role_id become optional and product_id is added.
ALTER TABLE works ALTER COLUMN task_id DROP NOT NULL;
ALTER TABLE works ALTER COLUMN role_id DROP NOT NULL;

ALTER TABLE works ADD COLUMN product_id UUID
    REFERENCES products(id) ON DELETE RESTRICT;
