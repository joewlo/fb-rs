-- 003_transactions.up.sql
CREATE TABLE transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    instrument_type TEXT NOT NULL,
    instrument_id TEXT NOT NULL,
    -- Transaction linking (DAG)
    parent_tx_id UUID REFERENCES transactions(id),
    root_tx_id UUID,
    link_type TEXT,
    link_depth INT NOT NULL DEFAULT 0,
    -- Attributes
    input_attributes JSONB NOT NULL DEFAULT '{}',
    derived_attributes JSONB,
    -- Enricher metadata
    enricher_name TEXT NOT NULL,
    enricher_version TEXT NOT NULL,
    contract_version TEXT,
    -- Status
    status TEXT NOT NULL DEFAULT 'submitted',
    idempotency_key UUID,
    error_reason TEXT,
    -- Metadata
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    posted_at TIMESTAMPTZ,
    CONSTRAINT tx_status_check CHECK (status IN ('submitted', 'enriched', 'validated', 'posted', 'failed', 'cancelled')),
    CONSTRAINT tx_link_depth_check CHECK (link_depth >= 0)
);

CREATE INDEX idx_tx_tenant ON transactions (tenant_id);
CREATE INDEX idx_tx_tenant_status ON transactions (tenant_id, status);
CREATE INDEX idx_tx_root ON transactions (tenant_id, root_tx_id);
CREATE INDEX idx_tx_parent ON transactions (tenant_id, parent_tx_id) WHERE parent_tx_id IS NOT NULL;
CREATE INDEX idx_tx_idempotency ON transactions (tenant_id, idempotency_key) WHERE idempotency_key IS NOT NULL;
CREATE INDEX idx_tx_instrument ON transactions (tenant_id, instrument_type, instrument_id);
CREATE INDEX idx_tx_posted_at ON transactions (tenant_id, posted_at) WHERE posted_at IS NOT NULL;
CREATE INDEX idx_tx_created_at ON transactions (tenant_id, created_at);
