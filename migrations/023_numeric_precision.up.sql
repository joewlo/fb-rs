-- 023_numeric_precision.up.sql
-- Remove precision limits on monetary columns. NUMERIC without constraints
-- stores up to 131,072 digits before the decimal and 16,383 after.
-- This handles crypto tokens with 22+ decimal places (e.g., SHIB, PEPE).

ALTER TABLE journal_entries ALTER COLUMN amount TYPE NUMERIC;
ALTER TABLE subledger_entries ALTER COLUMN quantity TYPE NUMERIC;
ALTER TABLE subledger_entries ALTER COLUMN price TYPE NUMERIC;
ALTER TABLE accounts ALTER COLUMN balance TYPE NUMERIC;
ALTER TABLE accounts ALTER COLUMN frozen_balance TYPE NUMERIC;
ALTER TABLE quantity_positions ALTER COLUMN quantity TYPE NUMERIC;
ALTER TABLE stock_records ALTER COLUMN quantity TYPE NUMERIC;
ALTER TABLE stock_records ALTER COLUMN cost_basis TYPE NUMERIC;
ALTER TABLE position_records ALTER COLUMN quantity_current TYPE NUMERIC;
ALTER TABLE position_records ALTER COLUMN quantity_safe_keeping TYPE NUMERIC;
ALTER TABLE position_records ALTER COLUMN quantity_segregated TYPE NUMERIC;
ALTER TABLE position_records ALTER COLUMN quantity_pending TYPE NUMERIC;
ALTER TABLE position_records ALTER COLUMN cost_basis TYPE NUMERIC;
ALTER TABLE position_records ALTER COLUMN realized_pnl TYPE NUMERIC;
ALTER TABLE position_records ALTER COLUMN unrealized_pnl TYPE NUMERIC;
ALTER TABLE position_records ALTER COLUMN last_price TYPE NUMERIC;
ALTER TABLE fee_schedules ALTER COLUMN min_amount TYPE NUMERIC;
ALTER TABLE fee_schedules ALTER COLUMN max_amount TYPE NUMERIC;
ALTER TABLE fee_calculations ALTER COLUMN basis_amount TYPE NUMERIC;
ALTER TABLE fee_calculations ALTER COLUMN fee_amount TYPE NUMERIC;
ALTER TABLE margin_parameters ALTER COLUMN min_margin_amount TYPE NUMERIC;
ALTER TABLE margin_snapshots ALTER COLUMN market_value TYPE NUMERIC;
ALTER TABLE margin_snapshots ALTER COLUMN loan_value TYPE NUMERIC;
ALTER TABLE margin_snapshots ALTER COLUMN margin_required TYPE NUMERIC;
ALTER TABLE margin_snapshots ALTER COLUMN margin_excess TYPE NUMERIC;
ALTER TABLE margin_snapshots ALTER COLUMN reference_price TYPE NUMERIC;
