-- 005_journal_entries.up.sql
-- Immutable journal entries (append-only). Each entry is one leg of a double-entry transaction.
CREATE TABLE journal_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    transaction_id UUID NOT NULL,
    entry_sequence INT NOT NULL,
    account_id UUID NOT NULL,
    amount NUMERIC(38,18) NOT NULL,
    currency TEXT NOT NULL,
    side TEXT NOT NULL,
    value_date DATE NOT NULL,
    narrative TEXT NOT NULL DEFAULT '',
    metadata JSONB NOT NULL DEFAULT '{}',
    posted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (transaction_id, entry_sequence),
    CONSTRAINT je_side_check CHECK (side IN ('DEBIT', 'CREDIT')),
    CONSTRAINT je_amount_check CHECK (amount > 0)
);

CREATE INDEX idx_je_transaction ON journal_entries (tenant_id, transaction_id);
CREATE INDEX idx_je_account_date ON journal_entries (tenant_id, account_id, value_date);
CREATE INDEX idx_je_account ON journal_entries (tenant_id, account_id);
CREATE INDEX idx_je_date ON journal_entries (tenant_id, value_date);
CREATE INDEX idx_je_posted_at ON journal_entries (tenant_id, posted_at);
