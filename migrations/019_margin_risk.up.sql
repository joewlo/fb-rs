-- 019_margin_risk.up.sql
-- Margin and loan value model: haircuts, collateral values, position-level risk metrics.
-- Each instrument has configurable margin parameters. Positions have computed loan values.

-- Instrument-level margin/risk parameters
CREATE TABLE margin_parameters (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    instrument_id TEXT NOT NULL,
    instrument_type TEXT NOT NULL,
    -- Haircut / margin rate (e.g., 0.15 = 15% haircut, 85% loan value)
    haircut_rate NUMERIC(8,6) NOT NULL DEFAULT 0.15,
    -- Concentration limit (max position as % of total portfolio)
    concentration_limit NUMERIC(8,6),
    -- Liquidity adjustment
    liquidity_haircut NUMERIC(8,6) NOT NULL DEFAULT 0,
    -- Counterparty risk adjustment
    counterparty_haircut NUMERIC(8,6) NOT NULL DEFAULT 0,
    -- Minimum margin requirement
    min_margin_amount NUMERIC(38,18),
    -- Contract reference for programmable margin calculation
    contract_name TEXT,
    contract_version TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, instrument_id)
);

CREATE INDEX idx_mp_tenant ON margin_parameters (tenant_id);
CREATE INDEX idx_mp_type ON margin_parameters (tenant_id, instrument_type);

-- Position-level margin/loan value snapshots
CREATE TABLE margin_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    account_id UUID NOT NULL,
    instrument_id TEXT NOT NULL,
    -- Market value of the position
    market_value NUMERIC(38,18) NOT NULL,
    -- Total haircut applied (sum of instrument + liquidity + counterparty)
    total_haircut NUMERIC(8,6) NOT NULL,
    -- Loan value = market_value × (1 - total_haircut)
    loan_value NUMERIC(38,18) NOT NULL,
    -- Margin required = min_margin or market_value × haircut
    margin_required NUMERIC(38,18) NOT NULL,
    -- Excess/Deficit = equity - margin_required
    margin_excess NUMERIC(38,18) NOT NULL DEFAULT 0,
    -- Reference price used
    reference_price NUMERIC(38,18),
    -- Currency
    currency TEXT NOT NULL DEFAULT 'USD',
    snapshot_date DATE NOT NULL DEFAULT CURRENT_DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, account_id, instrument_id, snapshot_date)
);

CREATE INDEX idx_ms_tenant ON margin_snapshots (tenant_id, snapshot_date);
CREATE INDEX idx_ms_account ON margin_snapshots (tenant_id, account_id);
