-- 011_posting_templates.up.sql
-- Posting templates: declarative double-entry templates used to generate journal entries.
CREATE TABLE posting_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    template_name TEXT NOT NULL,
    entries JSONB NOT NULL,
    version TEXT NOT NULL DEFAULT '1.0.0',
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (tenant_id, template_name, version)
);

CREATE INDEX idx_templates_tenant_name ON posting_templates (tenant_id, template_name) WHERE active = true;
