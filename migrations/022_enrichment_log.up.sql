-- 022_enrichment_log.up.sql
-- Full audit trail of every enrichment cycle step for every transaction.
-- Records what the contract computed, which fees applied, and all derived values.
-- Makes the enrichment process fully auditable and deterministic.
CREATE TABLE enrichment_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    transaction_id UUID NOT NULL,
    stage TEXT NOT NULL,
    step_name TEXT NOT NULL,
    step_order INT NOT NULL,
    -- What went in (snapshot of input attributes at this step)
    input_snapshot JSONB,
    -- What came out (the computed/derived values)
    output_snapshot JSONB,
    -- Which contract/enricher ran
    contract_name TEXT,
    contract_version TEXT,
    -- Duration in microseconds
    duration_us BIGINT,
    -- Errors if any
    error_message TEXT,
    -- Full metadata
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT el_stage_check CHECK (stage IN ('INGEST','VALIDATE','ENRICH','FEE_CALCULATION','GENERATE','CHECK','POST'))
);

CREATE INDEX idx_el_tx ON enrichment_log (tenant_id, transaction_id);
CREATE INDEX idx_el_stage ON enrichment_log (tenant_id, stage);
CREATE INDEX idx_el_contract ON enrichment_log (tenant_id, contract_name) WHERE contract_name IS NOT NULL;

-- Add enrichment_log JSONB to transactions for fast access to the full cycle
ALTER TABLE transactions ADD COLUMN IF NOT EXISTS enrichment_log JSONB;

-- Currency precision: store amounts as NUMERIC without loss
-- The journal_entries.amount column is already NUMERIC(38,18)
-- Add a comment documenting the precision guarantees
COMMENT ON TABLE journal_entries IS 'Immutable journal entries. Amounts stored as NUMERIC(38,18) for perfect decimal precision. Every transaction MUST have ΣDEBIT = ΣCREDIT — enforced at Go pipeline + batch writer + DB trigger layers.';
