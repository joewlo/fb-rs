-- 004_events.up.sql
-- Immutable event log (append-only). Events are never updated or deleted.
CREATE TABLE events (
    id BIGSERIAL,
    tenant_id UUID NOT NULL,
    event_id UUID NOT NULL,
    aggregate_type TEXT NOT NULL,
    aggregate_id UUID NOT NULL,
    event_type TEXT NOT NULL,
    event_data JSONB NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    version BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (aggregate_type, aggregate_id, version),
    UNIQUE (tenant_id, event_id)
);

CREATE INDEX idx_events_tenant ON events (tenant_id);
CREATE INDEX idx_events_tenant_created ON events (tenant_id, created_at);
CREATE INDEX idx_events_aggregate ON events (tenant_id, aggregate_type, aggregate_id, version);
CREATE INDEX idx_events_event_type ON events (tenant_id, event_type);
CREATE INDEX idx_events_metadata_key ON events USING GIN (metadata);
