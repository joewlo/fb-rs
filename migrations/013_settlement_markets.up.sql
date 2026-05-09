-- 013_settlement_markets.up.sql
CREATE TABLE settlement_markets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    market_code TEXT NOT NULL,
    market_name TEXT NOT NULL,
    currency TEXT NOT NULL,
    geo TEXT NOT NULL,
    settlement_cycle TEXT NOT NULL DEFAULT 'T+2',
    business_days TEXT[] NOT NULL DEFAULT ARRAY['Mon','Tue','Wed','Thu','Fri'],
    holidays JSONB NOT NULL DEFAULT '[]',
    cutoff_time TIME NOT NULL DEFAULT '16:00:00',
    timezone TEXT NOT NULL DEFAULT 'America/New_York',
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, market_code)
);

CREATE INDEX idx_markets_tenant ON settlement_markets (tenant_id);

-- Suspense/offset accounts — temporary holding accounts for unmatched or in-flight transactions
CREATE TABLE suspense_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    account_id UUID NOT NULL REFERENCES accounts(id),
    suspense_type TEXT NOT NULL,
    description TEXT,
    auto_resolve_rules JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, account_id, suspense_type),
    CONSTRAINT st_check CHECK (suspense_type IN ('UNMATCHED', 'TIMING', 'FX_MISMATCH', 'BREAK', 'GENERAL'))
);
