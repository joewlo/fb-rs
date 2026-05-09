-- 025_tax_lots.up.sql
-- Tax lots: the fundamental unit of tax accounting. Each purchase creates lots.
-- On sale, lots are consumed (FIFO by default) and realized gain/loss is computed.

CREATE TABLE tax_lots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    account_id UUID NOT NULL REFERENCES accounts(id),
    instrument_id TEXT NOT NULL,
    -- Purchase details
    acquire_date DATE NOT NULL,
    acquire_price NUMERIC NOT NULL,
    original_quantity NUMERIC NOT NULL,
    remaining_quantity NUMERIC NOT NULL DEFAULT 0,
    -- Cost basis
    cost_basis_total NUMERIC NOT NULL,
    cost_basis_per_unit NUMERIC NOT NULL,
    -- Method
    cost_method TEXT NOT NULL DEFAULT 'FIFO',
    -- Status
    status TEXT NOT NULL DEFAULT 'open',
    closed_date DATE,
    -- Tax jurisdiction
    jurisdiction TEXT NOT NULL DEFAULT 'US',
    -- Metadata
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT tl_status_check CHECK (status IN ('open', 'partial', 'closed')),
    CONSTRAINT tl_method_check CHECK (cost_method IN ('FIFO', 'LIFO', 'HIFO', 'SPECIFIC_ID', 'AVERAGE_COST'))
);

CREATE INDEX idx_tl_account_instrument ON tax_lots (tenant_id, account_id, instrument_id, acquire_date);
CREATE INDEX idx_tl_open ON tax_lots (tenant_id, account_id, instrument_id) WHERE status IN ('open', 'partial');
CREATE INDEX idx_tl_date ON tax_lots (tenant_id, acquire_date);

-- Tax lot consumption: records which lots were sold for each trade.
CREATE TABLE tax_lot_consumptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    transaction_id UUID NOT NULL,
    lot_id UUID NOT NULL REFERENCES tax_lots(id),
    quantity_consumed NUMERIC NOT NULL,
    cost_basis_consumed NUMERIC NOT NULL,
    proceeds NUMERIC NOT NULL,
    realized_gain NUMERIC NOT NULL,
    holding_period_days INT NOT NULL,
    tax_classification TEXT NOT NULL DEFAULT 'SHORT_TERM',
    tax_rate_applied NUMERIC(6,4),
    jurisdiction TEXT NOT NULL DEFAULT 'US',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT tlc_class_check CHECK (tax_classification IN ('SHORT_TERM', 'LONG_TERM', 'TAX_FREE', 'EXEMPT'))
);

CREATE INDEX idx_tlc_tx ON tax_lot_consumptions (tenant_id, transaction_id);
CREATE INDEX idx_tlc_lot ON tax_lot_consumptions (tenant_id, lot_id);

-- Tax jurisdictions and their rules.
CREATE TABLE tax_jurisdictions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    jurisdiction_code TEXT NOT NULL,
    jurisdiction_name TEXT NOT NULL,
    -- Capital gains
    short_term_rate NUMERIC(6,4) NOT NULL DEFAULT 0.3700,
    long_term_rate NUMERIC(6,4) NOT NULL DEFAULT 0.2000,
    long_term_threshold_days INT NOT NULL DEFAULT 365,
    -- Default cost basis method for this jurisdiction
    default_cost_method TEXT NOT NULL DEFAULT 'FIFO',
    -- Transaction tax
    transaction_tax_rate NUMERIC(8,6) NOT NULL DEFAULT 0,
    -- Withholding
    dividend_withholding_rate NUMERIC(6,4) NOT NULL DEFAULT 0.3000,
    treaty_withholding_rate NUMERIC(6,4),
    -- Wash sale rules
    wash_sale_window_days INT NOT NULL DEFAULT 30,
    wash_sale_enabled BOOLEAN NOT NULL DEFAULT true,
    -- Status
    status TEXT NOT NULL DEFAULT 'active',
    effective_from DATE NOT NULL DEFAULT CURRENT_DATE,
    metadata JSONB NOT NULL DEFAULT '{}',
    UNIQUE(tenant_id, jurisdiction_code),
    CONSTRAINT tj_cost_method_check CHECK (default_cost_method IN ('FIFO', 'LIFO', 'HIFO', 'SPECIFIC_ID', 'AVERAGE_COST'))
);
