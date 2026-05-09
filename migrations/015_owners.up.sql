-- 015_owners.up.sql
-- Owners: persons or entities that can own accounts. Attributes are extensible and programmable.
CREATE TABLE owners (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    owner_code TEXT NOT NULL,
    owner_type TEXT NOT NULL,
    name TEXT NOT NULL,
    -- Extensible attributes (schema-flexible, same philosophy as transaction attributes)
    attributes JSONB NOT NULL DEFAULT '{}',
    -- Contract reference for programmable attribute expansion
    contract_name TEXT,
    contract_version TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, owner_code),
    CONSTRAINT ot_check CHECK (owner_type IN ('PERSON', 'ENTITY', 'FUND', 'TRUST', 'DESK', 'CUSTODIAN'))
);

CREATE INDEX idx_owners_tenant ON owners (tenant_id);
CREATE INDEX idx_owners_type ON owners (tenant_id, owner_type);

-- Account ownership: many-to-many with ownership percentage and effective dates
CREATE TABLE account_ownership (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    owner_id UUID NOT NULL REFERENCES owners(id),
    account_id UUID NOT NULL REFERENCES accounts(id),
    ownership_pct NUMERIC(5,2) NOT NULL DEFAULT 100.00,
    effective_from DATE NOT NULL DEFAULT CURRENT_DATE,
    effective_to DATE,
    role TEXT NOT NULL DEFAULT 'BENEFICIAL',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, owner_id, account_id, effective_from),
    CONSTRAINT ao_role_check CHECK (role IN ('BENEFICIAL', 'LEGAL', 'NOMINEE', 'CUSTODIAN', 'TRADER'))
);

CREATE INDEX idx_ao_owner ON account_ownership (tenant_id, owner_id);
CREATE INDEX idx_ao_account ON account_ownership (tenant_id, account_id);
CREATE INDEX idx_ao_effective ON account_ownership (tenant_id, effective_from) WHERE effective_to IS NULL;
