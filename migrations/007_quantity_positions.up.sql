-- 007_quantity_positions.up.sql
-- Typed quantity position tracking. Positions are maintained per account, instrument, and quantity type.
CREATE TABLE quantity_positions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    account_id UUID NOT NULL,
    instrument_id TEXT NOT NULL,
    quantity_type TEXT NOT NULL,
    quantity NUMERIC(38,18) NOT NULL DEFAULT 0,
    version BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, account_id, instrument_id, quantity_type),
    CONSTRAINT qp_quantity_type_check CHECK (quantity_type IN (
        'current', 'traded', 'safe_keeping', 'segregated', 'frozen',
        'available', 'pending_settlement', 'pledged', 'recalled'
    ))
);

CREATE INDEX idx_qp_tenant_account ON quantity_positions (tenant_id, account_id);
CREATE INDEX idx_qp_instrument ON quantity_positions (tenant_id, instrument_id);
CREATE INDEX idx_qp_instrument_type ON quantity_positions (tenant_id, instrument_id, quantity_type);
