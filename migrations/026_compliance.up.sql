-- 026_compliance.up.sql
-- Compliance: sanctions screening, AML, KYC, financial crimes detection.
-- Runs during the VALIDATE stage of the enrichment pipeline.

-- Sanctions lists: OFAC SDN, UN, EU, UK, etc.
CREATE TABLE sanctions_lists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    list_name TEXT NOT NULL,
    list_authority TEXT NOT NULL,
    list_version TEXT NOT NULL,
    effective_date DATE NOT NULL DEFAULT CURRENT_DATE,
    entity_count INT NOT NULL DEFAULT 0,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, list_name, list_version)
);

-- Sanctioned entities: individuals, organizations, vessels, etc.
CREATE TABLE sanctioned_entities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    list_id UUID NOT NULL REFERENCES sanctions_lists(id),
    entity_type TEXT NOT NULL DEFAULT 'INDIVIDUAL',
    full_name TEXT NOT NULL,
    aliases JSONB NOT NULL DEFAULT '[]',
    identifiers JSONB NOT NULL DEFAULT '{}',
    sanctions_program TEXT,
    date_listed DATE,
    risk_level TEXT NOT NULL DEFAULT 'HIGH',
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT se_type_check CHECK (entity_type IN ('INDIVIDUAL', 'ENTITY', 'VESSEL', 'AIRCRAFT', 'DIGITAL_WALLET'))
);

CREATE INDEX idx_se_name ON sanctioned_entities (tenant_id, full_name);
CREATE INDEX idx_se_name_trgm ON sanctioned_entities USING gin (full_name gin_trgm_ops);
CREATE INDEX idx_se_list ON sanctioned_entities (tenant_id, list_id);

-- Compliance alerts: generated when a check triggers a warning.
CREATE TABLE compliance_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    transaction_id UUID,
    alert_type TEXT NOT NULL,
    alert_severity TEXT NOT NULL DEFAULT 'MEDIUM',
    alert_message TEXT NOT NULL,
    matched_entity TEXT,
    match_score NUMERIC(5,2),
    source TEXT,
    status TEXT NOT NULL DEFAULT 'OPEN',
    resolved_by TEXT,
    resolved_at TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT ca_severity_check CHECK (alert_severity IN ('LOW', 'MEDIUM', 'HIGH', 'CRITICAL')),
    CONSTRAINT ca_type_check CHECK (alert_type IN (
        'SANCTIONS_MATCH', 'PEP_MATCH', 'ADVERSE_MEDIA',
        'AML_VELOCITY', 'AML_STRUCTURING', 'AML_UNUSUAL_VOLUME',
        'KYC_EXPIRED', 'KYC_MISSING',
        'RISK_LIMIT_BREACH', 'GEO_RESTRICTION',
        'INSIDER_TRADING_FLAG', 'MARKET_MANIPULATION_FLAG'
    ))
);

CREATE INDEX idx_ca_tx ON compliance_alerts (tenant_id, transaction_id) WHERE transaction_id IS NOT NULL;
CREATE INDEX idx_ca_status ON compliance_alerts (tenant_id, status);
CREATE INDEX idx_ca_created ON compliance_alerts (tenant_id, created_at);

-- AML rules: configurable thresholds for money laundering detection.
CREATE TABLE aml_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    rule_code TEXT NOT NULL,
    rule_name TEXT NOT NULL,
    rule_type TEXT NOT NULL,
    -- Parameters
    params JSONB NOT NULL DEFAULT '{}',
    -- Scope
    instrument_type TEXT,
    jurisdiction TEXT,
    -- Action
    action TEXT NOT NULL DEFAULT 'ALERT',
    severity TEXT NOT NULL DEFAULT 'MEDIUM',
    status TEXT NOT NULL DEFAULT 'active',
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, rule_code),
    CONSTRAINT aml_action_check CHECK (action IN ('ALERT', 'BLOCK', 'FLAG', 'LOG')),
    CONSTRAINT aml_type_check CHECK (rule_type IN (
        'VELOCITY', 'VOLUME', 'STRUCTURING', 'COUNTERPARTY_CONCENTRATION',
        'GEO_RESTRICTION', 'NESTED_OWNERSHIP', 'ROUND_TRIP', 'LAYERING'
    ))
);
