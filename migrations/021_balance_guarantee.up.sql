-- 021_balance_guarantee.up.sql
-- Balance guarantee: debits MUST equal credits. Enforced at every layer.

CREATE OR REPLACE FUNCTION check_transaction_balance(tx_id UUID, td UUID)
RETURNS BOOLEAN AS $$
DECLARE
    total_d NUMERIC;
    total_c NUMERIC;
BEGIN
    SELECT
        COALESCE(SUM(CASE WHEN side='DEBIT' THEN amount ELSE 0 END), 0),
        COALESCE(SUM(CASE WHEN side='CREDIT' THEN amount ELSE 0 END), 0)
    INTO total_d, total_c
    FROM journal_entries
    WHERE transaction_id = tx_id AND tenant_id = td;
    RETURN total_d = total_c;
END;
$$ LANGUAGE plpgsql IMMUTABLE;
