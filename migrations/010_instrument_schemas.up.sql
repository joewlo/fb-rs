-- 010_instrument_schemas.up.sql
-- Instrument type schemas: declare input attributes, calculated attributes, and posting rules per instrument type.
CREATE TABLE instrument_schemas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    instrument_type TEXT NOT NULL,
    schema_data JSONB NOT NULL,
    enricher_name TEXT NOT NULL,
    version TEXT NOT NULL DEFAULT '1.0.0',
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, instrument_type, version)
);

CREATE INDEX idx_schemas_tenant_type ON instrument_schemas (tenant_id, instrument_type) WHERE active = true;
