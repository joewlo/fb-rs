-- 001_tenants.up.sql
CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    short_code TEXT UNIQUE NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT tenants_status_check CHECK (status IN ('active', 'inactive', 'suspended'))
);

CREATE INDEX idx_tenants_short_code ON tenants (short_code);
CREATE INDEX idx_tenants_status ON tenants (status);
