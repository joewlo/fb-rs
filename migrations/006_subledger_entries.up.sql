-- 006_subledger_entries.up.sql
-- Subledger entries (append-only). Contain subledger-specific detail alongside a journal entry reference.
CREATE TABLE subledger_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    transaction_id UUID NOT NULL,
    subledger_type TEXT NOT NULL,
    journal_entry_id UUID NOT NULL,
    instrument_id TEXT,
    quantity NUMERIC(38,18),
    quantity_type TEXT,
    price NUMERIC(38,18),
    counterparty TEXT,
    trade_date DATE,
    settle_date DATE,
    metadata JSONB NOT NULL DEFAULT '{}',
    CONSTRAINT sle_subledger_check CHECK (subledger_type IN ('TRADING', 'CASH', 'PNL', 'SETTLEMENT', 'POSITION')),
    FOREIGN KEY (journal_entry_id) REFERENCES journal_entries(id)
);

CREATE INDEX idx_sle_transaction ON subledger_entries (tenant_id, transaction_id);
CREATE INDEX idx_sle_subledger ON subledger_entries (tenant_id, subledger_type);
CREATE INDEX idx_sle_instrument ON subledger_entries (tenant_id, instrument_id) WHERE instrument_id IS NOT NULL;
CREATE INDEX idx_sle_counterparty ON subledger_entries (tenant_id, counterparty) WHERE counterparty IS NOT NULL;
CREATE INDEX idx_sle_settle_date ON subledger_entries (tenant_id, settle_date) WHERE settle_date IS NOT NULL;
