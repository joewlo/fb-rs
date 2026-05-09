-- 024_price_feed.up.sql
-- Streaming price feed: stores real-time market prices for mark-to-market.
-- Populated by the PriceController from Kafka topic "market.prices".
CREATE TABLE price_feed (
    id BIGSERIAL PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    instrument_id TEXT NOT NULL,
    price_type TEXT NOT NULL DEFAULT 'last',
    price NUMERIC NOT NULL,
    bid NUMERIC,
    ask NUMERIC,
    volume_24h NUMERIC,
    source TEXT NOT NULL DEFAULT 'stream',
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_pf_latest ON price_feed (tenant_id, instrument_id, price_type, timestamp DESC);

-- Position cost basis: tracks the average cost per position for realized P&L.
-- Updated on each trade; used for P&L calculation on sell.
CREATE TABLE position_cost_basis (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    account_id UUID NOT NULL,
    instrument_id TEXT NOT NULL,
    quantity NUMERIC NOT NULL DEFAULT 0,
    cost_basis_total NUMERIC NOT NULL DEFAULT 0,
    cost_basis_per_unit NUMERIC,
    total_realized_pnl NUMERIC NOT NULL DEFAULT 0,
    total_unrealized_pnl NUMERIC NOT NULL DEFAULT 0,
    last_price NUMERIC,
    version BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, account_id, instrument_id)
);

CREATE INDEX idx_pcb_account ON position_cost_basis (tenant_id, account_id);
CREATE INDEX idx_pcb_instrument ON position_cost_basis (tenant_id, instrument_id);

-- Daily P&L snapshots: records P&L per desk per day for reporting.
CREATE TABLE pnl_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    desk TEXT NOT NULL,
    instrument_id TEXT,
    snapshot_date DATE NOT NULL,
    realized_pnl NUMERIC NOT NULL DEFAULT 0,
    unrealized_pnl NUMERIC NOT NULL DEFAULT 0,
    commission_paid NUMERIC NOT NULL DEFAULT 0,
    fee_paid NUMERIC NOT NULL DEFAULT 0,
    interest_accrued NUMERIC NOT NULL DEFAULT 0,
    gross_pnl NUMERIC NOT NULL DEFAULT 0,
    net_pnl NUMERIC NOT NULL DEFAULT 0,
    trade_count INT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, desk, instrument_id, snapshot_date)
);

CREATE INDEX idx_pnl_desk_date ON pnl_snapshots (tenant_id, desk, snapshot_date);
CREATE INDEX idx_pnl_date ON pnl_snapshots (tenant_id, snapshot_date);
