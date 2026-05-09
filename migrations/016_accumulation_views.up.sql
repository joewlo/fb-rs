-- 016_accumulation_views.up.sql
-- Stock record: all accounts holding a given instrument, by quantity type
CREATE TABLE stock_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    instrument_id TEXT NOT NULL,
    quantity_type TEXT NOT NULL,
    account_id UUID NOT NULL,
    quantity NUMERIC(38,18) NOT NULL DEFAULT 0,
    cost_basis NUMERIC(38,18),
    market_code TEXT,
    version BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, instrument_id, quantity_type, account_id)
);

CREATE INDEX idx_sr_instrument ON stock_records (tenant_id, instrument_id);
CREATE INDEX idx_sr_account ON stock_records (tenant_id, account_id);

-- Position view: all instruments held by a given account
CREATE TABLE position_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    account_id UUID NOT NULL,
    instrument_id TEXT NOT NULL,
    quantity_current NUMERIC(38,18) NOT NULL DEFAULT 0,
    quantity_safe_keeping NUMERIC(38,18) NOT NULL DEFAULT 0,
    quantity_segregated NUMERIC(38,18) NOT NULL DEFAULT 0,
    quantity_pending NUMERIC(38,18) NOT NULL DEFAULT 0,
    cost_basis NUMERIC(38,18),
    realized_pnl NUMERIC(38,18) NOT NULL DEFAULT 0,
    unrealized_pnl NUMERIC(38,18) NOT NULL DEFAULT 0,
    last_price NUMERIC(38,18),
    version BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, account_id, instrument_id)
);

CREATE INDEX idx_pr_account ON position_records (tenant_id, account_id);
CREATE INDEX idx_pr_instrument ON position_records (tenant_id, instrument_id);
