-- 009_idempotency_keys.up.sql
-- Tracks idempotency keys to prevent duplicate transaction processing.
CREATE TABLE idempotency_keys (
    tenant_id UUID NOT NULL,
    idempotency_key UUID NOT NULL,
    event_id UUID,
    result JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (tenant_id, idempotency_key)
);
