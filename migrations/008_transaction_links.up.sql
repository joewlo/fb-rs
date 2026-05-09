-- 008_transaction_links.up.sql
-- DAG edges between parent and child transactions.
CREATE TABLE transaction_links (
    parent_tx_id UUID NOT NULL REFERENCES transactions(id),
    child_tx_id UUID NOT NULL REFERENCES transactions(id),
    tenant_id UUID NOT NULL,
    link_type TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (tenant_id, parent_tx_id, child_tx_id),
    CONSTRAINT tx_links_no_self CHECK (parent_tx_id <> child_tx_id)
);

CREATE INDEX idx_tx_links_child ON transaction_links (tenant_id, child_tx_id);
CREATE INDEX idx_tx_links_type ON transaction_links (tenant_id, link_type);
