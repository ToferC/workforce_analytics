-- diesel:disable-transaction
-- ALTER TYPE ... ADD VALUE cannot run inside a transaction on PostgreSQL < 12.

-- Ensure product_agile_and_delivery exists in the PG type.
-- (Earlier versions of the 2026-06-04 migration added 'agile_and_delivery' instead.)
ALTER TYPE skill_domain ADD VALUE IF NOT EXISTS 'product_agile_and_delivery';

-- ── Remap legacy values that no longer have Rust enum variants ──────────────
--
-- Original 13-variant type     →  current SkillDomain variant
-- ─────────────────────────────────────────────────────────────
-- information_technology       →  cloud_platform_dev_ops
-- management                   →  people_and_organisational_leadership
-- leadership                   →  people_and_organisational_leadership
-- human_resources              →  corporate_services
-- finance                      →  corporate_services
-- communications               →  corporate_services
-- administration               →  corporate_services
--
-- Renamed variants (2026-06-04 migration, pre-edit version)
-- agile_and_delivery           →  product_agile_and_delivery
-- product_management           →  product_agile_and_delivery

UPDATE org_tiers SET primary_domain = 'cloud_platform_dev_ops'              WHERE primary_domain::text = 'information_technology';
UPDATE org_tiers SET primary_domain = 'people_and_organisational_leadership' WHERE primary_domain::text IN ('management', 'leadership');
UPDATE org_tiers SET primary_domain = 'corporate_services'                   WHERE primary_domain::text IN ('human_resources', 'finance', 'communications', 'administration');
UPDATE org_tiers SET primary_domain = 'product_agile_and_delivery'           WHERE primary_domain::text IN ('agile_and_delivery', 'product_management');

UPDATE teams SET primary_domain = 'cloud_platform_dev_ops'              WHERE primary_domain::text = 'information_technology';
UPDATE teams SET primary_domain = 'people_and_organisational_leadership' WHERE primary_domain::text IN ('management', 'leadership');
UPDATE teams SET primary_domain = 'corporate_services'                   WHERE primary_domain::text IN ('human_resources', 'finance', 'communications', 'administration');
UPDATE teams SET primary_domain = 'product_agile_and_delivery'           WHERE primary_domain::text IN ('agile_and_delivery', 'product_management');

UPDATE skills SET domain = 'cloud_platform_dev_ops'              WHERE domain::text = 'information_technology';
UPDATE skills SET domain = 'people_and_organisational_leadership' WHERE domain::text IN ('management', 'leadership');
UPDATE skills SET domain = 'corporate_services'                   WHERE domain::text IN ('human_resources', 'finance', 'communications', 'administration');
UPDATE skills SET domain = 'product_agile_and_delivery'           WHERE domain::text IN ('agile_and_delivery', 'product_management');

UPDATE capabilities SET domain = 'cloud_platform_dev_ops'              WHERE domain::text = 'information_technology';
UPDATE capabilities SET domain = 'people_and_organisational_leadership' WHERE domain::text IN ('management', 'leadership');
UPDATE capabilities SET domain = 'corporate_services'                   WHERE domain::text IN ('human_resources', 'finance', 'communications', 'administration');
UPDATE capabilities SET domain = 'product_agile_and_delivery'           WHERE domain::text IN ('agile_and_delivery', 'product_management');

UPDATE tasks SET domain = 'cloud_platform_dev_ops'              WHERE domain::text = 'information_technology';
UPDATE tasks SET domain = 'people_and_organisational_leadership' WHERE domain::text IN ('management', 'leadership');
UPDATE tasks SET domain = 'corporate_services'                   WHERE domain::text IN ('human_resources', 'finance', 'communications', 'administration');
UPDATE tasks SET domain = 'product_agile_and_delivery'           WHERE domain::text IN ('agile_and_delivery', 'product_management');

UPDATE works SET domain = 'cloud_platform_dev_ops'              WHERE domain::text = 'information_technology';
UPDATE works SET domain = 'people_and_organisational_leadership' WHERE domain::text IN ('management', 'leadership');
UPDATE works SET domain = 'corporate_services'                   WHERE domain::text IN ('human_resources', 'finance', 'communications', 'administration');
UPDATE works SET domain = 'product_agile_and_delivery'           WHERE domain::text IN ('agile_and_delivery', 'product_management');

UPDATE requirements SET domain = 'cloud_platform_dev_ops'              WHERE domain::text = 'information_technology';
UPDATE requirements SET domain = 'people_and_organisational_leadership' WHERE domain::text IN ('management', 'leadership');
UPDATE requirements SET domain = 'corporate_services'                   WHERE domain::text IN ('human_resources', 'finance', 'communications', 'administration');
UPDATE requirements SET domain = 'product_agile_and_delivery'           WHERE domain::text IN ('agile_and_delivery', 'product_management');
