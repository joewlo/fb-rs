-- 002_accounts.up.sql
CREATE TABLE accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    geo TEXT NOT NULL,
    account_code TEXT NOT NULL,
    account_name TEXT NOT NULL,
    account_type TEXT NOT NULL,
    subledger_type TEXT,
    currency TEXT NOT NULL DEFAULT 'USD',
    balance NUMERIC(38,18) NOT NULL DEFAULT 0,
    frozen_balance NUMERIC(38,18) NOT NULL DEFAULT 0,
    version BIGINT NOT NULL DEFAULT 0,
    sequence_number BIGINT NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'active',
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, geo, account_code),
    CONSTRAINT accounts_status_check CHECK (status IN ('active', 'frozen', 'closed')),
    CONSTRAINT accounts_type_check CHECK (account_type IN ('ASSET', 'LIABILITY', 'EQUITY', 'INCOME', 'EXPENSE'))
);

CREATE INDEX idx_accounts_tenant ON accounts (tenant_id);
CREATE INDEX idx_accounts_tenant_geo ON accounts (tenant_id, geo);
CREATE INDEX idx_accounts_type ON accounts (account_type);
CREATE INDEX idx_accounts_subledger ON accounts (tenant_id, subledger_type) WHERE subledger_type IS NOT NULL;
