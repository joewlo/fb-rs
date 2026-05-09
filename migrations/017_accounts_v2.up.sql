-- 017_accounts_v2.up.sql
-- Drop the unique constraint to rebuild it with the new columns
ALTER TABLE accounts DROP CONSTRAINT IF EXISTS accounts_tenant_id_geo_account_code_key;

-- Add display columns and extensible attributes
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS display_name TEXT;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS display_code TEXT;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS attributes JSONB NOT NULL DEFAULT '{}';
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS contract_name TEXT;
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS contract_version TEXT;

-- Populate display_name from account_name, display_code from account_code
UPDATE accounts SET display_name = account_name WHERE display_name IS NULL;
UPDATE accounts SET display_code = account_code WHERE display_code IS NULL;

-- Re-add unique constraint
ALTER TABLE accounts ADD CONSTRAINT accounts_tenant_id_geo_account_code_key UNIQUE (tenant_id, geo, account_code);
