-- 014_instruments.up.sql
-- Security master: instruments with extensible, programmable attributes
CREATE TABLE instruments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    instrument_id TEXT NOT NULL,
    instrument_type TEXT NOT NULL,
    name TEXT NOT NULL,
    issuer TEXT,
    currency TEXT NOT NULL DEFAULT 'USD',
    market_code TEXT,
    -- Core fixed attributes
    attributes JSONB NOT NULL DEFAULT '{}',
    -- Derived/computed attributes (populated by enrichment)
    computed_attributes JSONB NOT NULL DEFAULT '{}',
    -- Contract reference for programmable behavior
    contract_name TEXT,
    contract_version TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, instrument_id)
);

CREATE INDEX idx_instruments_tenant ON instruments (tenant_id);
CREATE INDEX idx_instruments_type ON instruments (tenant_id, instrument_type);
CREATE INDEX idx_instruments_issuer ON instruments (tenant_id, issuer) WHERE issuer IS NOT NULL;
CREATE INDEX idx_instruments_market ON instruments (tenant_id, market_code) WHERE market_code IS NOT NULL;
