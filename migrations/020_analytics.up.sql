-- 020_analytics.up.sql
-- Auto-analytics: per instrument/account/desk/day aggregated statistics.
-- Populated incrementally as transactions are posted. Supports max, min, avg, count, sum, stddev.

CREATE TABLE analytics_instrument (
    tenant_id UUID NOT NULL,
    instrument_id TEXT NOT NULL,
    trade_date DATE NOT NULL,
    -- Price stats
    price_high NUMERIC(38,18) NOT NULL DEFAULT 0,
    price_low NUMERIC(38,18) NOT NULL DEFAULT 0,
    price_open NUMERIC(38,18),
    price_close NUMERIC(38,18),
    price_avg NUMERIC(38,18) NOT NULL DEFAULT 0,
    price_vwap NUMERIC(38,18),           -- volume-weighted average price
    -- Volume stats
    volume_sum NUMERIC(38,18) NOT NULL DEFAULT 0,
    volume_count INT NOT NULL DEFAULT 0,
    volume_max NUMERIC(38,18) NOT NULL DEFAULT 0,
    volume_min NUMERIC(38,18) NOT NULL DEFAULT 0,
    -- Notional stats
    notional_sum NUMERIC(38,18) NOT NULL DEFAULT 0,
    -- Fee stats
    fee_sum NUMERIC(38,18) NOT NULL DEFAULT 0,
    -- Slope (price change over period)
    price_slope NUMERIC(38,18),
    -- Standard deviation (sample)
    price_stddev NUMERIC(38,18),
    -- Count
    trade_count INT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (tenant_id, instrument_id, trade_date)
);

CREATE TABLE analytics_desk (
    tenant_id UUID NOT NULL,
    desk TEXT NOT NULL,
    trade_date DATE NOT NULL,
    notional_sum NUMERIC(38,18) NOT NULL DEFAULT 0,
    fee_sum NUMERIC(38,18) NOT NULL DEFAULT 0,
    commission_sum NUMERIC(38,18) NOT NULL DEFAULT 0,
    trade_count INT NOT NULL DEFAULT 0,
    pnl_gross NUMERIC(38,18) NOT NULL DEFAULT 0,
    pnl_net NUMERIC(38,18) NOT NULL DEFAULT 0,
    instrument_count INT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (tenant_id, desk, trade_date)
);

CREATE TABLE analytics_account (
    tenant_id UUID NOT NULL,
    account_id UUID NOT NULL,
    trade_date DATE NOT NULL,
    debit_sum NUMERIC(38,18) NOT NULL DEFAULT 0,
    credit_sum NUMERIC(38,18) NOT NULL DEFAULT 0,
    entry_count INT NOT NULL DEFAULT 0,
    net_flow NUMERIC(38,18) NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (tenant_id, account_id, trade_date)
);

CREATE INDEX idx_ai_date ON analytics_instrument (tenant_id, trade_date);
CREATE INDEX idx_ad_desk ON analytics_desk (tenant_id, desk);
CREATE INDEX idx_aa_account ON analytics_account (tenant_id, account_id);
