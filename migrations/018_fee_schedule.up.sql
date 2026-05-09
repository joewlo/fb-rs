-- 018_fee_schedule.up.sql
-- Programmable fee schedule: per tenant, market, instrument type, and counterparty.
-- Fees can be flat, percentage, or tiered. They stack and can be enabled/disabled.
CREATE TABLE fee_schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    fee_code TEXT NOT NULL,
    fee_name TEXT NOT NULL,
    fee_type TEXT NOT NULL,
    fee_category TEXT NOT NULL,
    -- Scope: which trades this fee applies to
    instrument_type TEXT,
    market_code TEXT,
    counterparty TEXT,
    -- Calculation: how the fee is computed
    calc_method TEXT NOT NULL DEFAULT 'percentage',
    calc_config JSONB NOT NULL DEFAULT '{}',
    -- Limits
    min_amount NUMERIC(38,18),
    max_amount NUMERIC(38,18),
    -- Currency
    currency TEXT NOT NULL DEFAULT 'USD',
    -- Priority: lower number = applied first. Multiple fees of same category stack.
    priority INT NOT NULL DEFAULT 100,
    status TEXT NOT NULL DEFAULT 'active',
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, fee_code),
    CONSTRAINT fs_calc CHECK (calc_method IN ('flat', 'percentage', 'bps', 'tiered', 'per_unit', 'contract')),
    CONSTRAINT fs_category CHECK (fee_category IN (
        'COMMISSION', 'EXCHANGE', 'CLEARING', 'REGULATORY', 'STAMP_DUTY',
        'CUSTODY', 'SETTLEMENT', 'TRANSACTION_TAX', 'BROKERAGE', 'SPREAD', 'OTHER'
    ))
);

CREATE INDEX idx_fees_tenant ON fee_schedules (tenant_id);
CREATE INDEX idx_fees_scope ON fee_schedules (tenant_id, instrument_type, market_code) WHERE instrument_type IS NOT NULL;

-- Fee calculation results logged per transaction
CREATE TABLE fee_calculations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    transaction_id UUID NOT NULL,
    fee_schedule_id UUID REFERENCES fee_schedules(id),
    fee_code TEXT NOT NULL,
    fee_category TEXT NOT NULL,
    calc_method TEXT NOT NULL,
    basis_amount NUMERIC(38,18) NOT NULL,
    fee_amount NUMERIC(38,18) NOT NULL,
    currency TEXT NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    calculated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_fc_tx ON fee_calculations (tenant_id, transaction_id);
