-- Add new skill domain enum variants for the consolidated digital & technology framework.
-- PostgreSQL does not support removing enum values, so old variants remain in the type
-- but are no longer referenced by the application.  Existing rows that still carry an
-- old value will need a data migration before those variants can be considered fully
-- retired.

ALTER TYPE skill_domain ADD VALUE IF NOT EXISTS 'software_engineering';
ALTER TYPE skill_domain ADD VALUE IF NOT EXISTS 'cloud_platform_dev_ops';
ALTER TYPE skill_domain ADD VALUE IF NOT EXISTS 'data_analytics_and_ai';
ALTER TYPE skill_domain ADD VALUE IF NOT EXISTS 'cyber_security';
ALTER TYPE skill_domain ADD VALUE IF NOT EXISTS 'product_management';
ALTER TYPE skill_domain ADD VALUE IF NOT EXISTS 'agile_and_delivery';
ALTER TYPE skill_domain ADD VALUE IF NOT EXISTS 'user_experience';
ALTER TYPE skill_domain ADD VALUE IF NOT EXISTS 'procurement_and_vendor_management';
ALTER TYPE skill_domain ADD VALUE IF NOT EXISTS 'people_and_organisational_leadership';
ALTER TYPE skill_domain ADD VALUE IF NOT EXISTS 'governance';
ALTER TYPE skill_domain ADD VALUE IF NOT EXISTS 'corporate_services';
