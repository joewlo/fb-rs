-- 012_contracts.up.sql
-- WASM contracts: stored WASM bytecode for user-defined enrichment contracts.
CREATE TABLE contracts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    name TEXT NOT NULL,
    instrument_type TEXT NOT NULL,
    wasm_bytes BYTEA NOT NULL,
    version TEXT NOT NULL DEFAULT '1.0.0',
    status TEXT NOT NULL DEFAULT 'draft',
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, name, version),
    CONSTRAINT contracts_status_check CHECK (status IN ('draft', 'active', 'archived', 'failed'))
);

CREATE INDEX idx_contracts_tenant_name ON contracts (tenant_id, name);
CREATE INDEX idx_contracts_active ON contracts (tenant_id, instrument_type) WHERE status = 'active';
