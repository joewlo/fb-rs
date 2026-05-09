use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::kernel::*;

#[derive(Debug, Clone, FromRow)]
struct TaxLotRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub account_id: Uuid,
    pub instrument_id: String,
    pub acquire_date: NaiveDate,
    pub acquire_price: Decimal,
    pub original_quantity: Decimal,
    pub remaining_quantity: Decimal,
    pub cost_basis_total: Decimal,
    pub cost_basis_per_unit: Decimal,
    pub cost_method: String,
    pub status: String,
    pub closed_date: Option<NaiveDate>,
    pub jurisdiction: String,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
struct TaxLotConsumptionRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub transaction_id: Uuid,
    pub lot_id: Uuid,
    pub quantity_consumed: Decimal,
    pub cost_basis_consumed: Decimal,
    pub proceeds: Decimal,
    pub realized_gain: Decimal,
    pub holding_period_days: i32,
    pub tax_classification: String,
    pub tax_rate_applied: Option<Decimal>,
    pub jurisdiction: String,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
struct TaxJurisdictionRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub jurisdiction_code: String,
    pub jurisdiction_name: String,
    pub short_term_rate: Decimal,
    pub long_term_rate: Decimal,
    pub long_term_threshold_days: i32,
    pub default_cost_method: String,
    pub transaction_tax_rate: Decimal,
    pub dividend_withholding_rate: Decimal,
    pub treaty_withholding_rate: Option<Decimal>,
    pub wash_sale_window_days: i32,
    pub wash_sale_enabled: bool,
    pub status: String,
    pub effective_from_date: NaiveDate,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct TaxLot {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub account_id: Uuid,
    pub instrument_id: String,
    pub acquire_date: NaiveDate,
    pub acquire_price: Decimal,
    pub original_quantity: Decimal,
    pub remaining_quantity: Decimal,
    pub cost_basis_total: Decimal,
    pub cost_basis_per_unit: Decimal,
    pub cost_method: String,
    pub status: String,
    pub closed_date: Option<NaiveDate>,
    pub jurisdiction: String,
}

#[derive(Debug, Clone)]
pub struct LotConsumption {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub transaction_id: Uuid,
    pub lot_id: Uuid,
    pub quantity_consumed: Decimal,
    pub cost_basis_consumed: Decimal,
    pub proceeds: Decimal,
    pub realized_gain: Decimal,
    pub holding_period_days: i32,
    pub tax_classification: String,
    pub tax_rate_applied: Option<Decimal>,
    pub jurisdiction: String,
}

#[derive(Debug, Clone)]
pub struct TaxComputation {
    pub short_term_gain: Decimal,
    pub long_term_gain: Decimal,
    pub total_gain: Decimal,
    pub short_term_tax: Decimal,
    pub long_term_tax: Decimal,
    pub total_tax: Decimal,
    pub effective_rate: Decimal,
    pub jurisdiction_code: String,
}

fn map_db_err(e: sqlx::Error) -> FbError {
    FbError::pipeline("DB", "tax", e)
}

impl From<TaxLotRow> for TaxLot {
    fn from(r: TaxLotRow) -> Self {
        TaxLot {
            id: r.id, tenant_id: r.tenant_id, account_id: r.account_id,
            instrument_id: r.instrument_id, acquire_date: r.acquire_date,
            acquire_price: r.acquire_price, original_quantity: r.original_quantity,
            remaining_quantity: r.remaining_quantity, cost_basis_total: r.cost_basis_total,
            cost_basis_per_unit: r.cost_basis_per_unit, cost_method: r.cost_method,
            status: r.status, closed_date: r.closed_date, jurisdiction: r.jurisdiction,
        }
    }
}

impl From<TaxLotConsumptionRow> for LotConsumption {
    fn from(r: TaxLotConsumptionRow) -> Self {
        LotConsumption {
            id: r.id, tenant_id: r.tenant_id, transaction_id: r.transaction_id,
            lot_id: r.lot_id, quantity_consumed: r.quantity_consumed,
            cost_basis_consumed: r.cost_basis_consumed, proceeds: r.proceeds,
            realized_gain: r.realized_gain, holding_period_days: r.holding_period_days,
            tax_classification: r.tax_classification, tax_rate_applied: r.tax_rate_applied,
            jurisdiction: r.jurisdiction,
        }
    }
}

pub struct TaxEngine {
    pool: PgPool,
}

impl TaxEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_lot(
        &self, tenant_id: Uuid, account_id: Uuid, instrument_id: &str,
        acquire_date: NaiveDate, acquire_price: Decimal, quantity: Decimal,
        cost_method: &str, jurisdiction: &str,
    ) -> Result<TaxLot, FbError> {
        let cost_basis_total = acquire_price * quantity;
        let cost_basis_per_unit = acquire_price;
        let row: TaxLotRow = sqlx::query_as(
            "INSERT INTO tax_lots (tenant_id, account_id, instrument_id, acquire_date, \
             acquire_price, original_quantity, remaining_quantity, cost_basis_total, \
             cost_basis_per_unit, cost_method, status, jurisdiction) \
             VALUES ($1, $2, $3, $4, $5, $6, $6, $7, $8, $9, 'open', $10) \
             RETURNING id, tenant_id, account_id, instrument_id, acquire_date, acquire_price, \
             original_quantity, remaining_quantity, cost_basis_total, cost_basis_per_unit, \
             cost_method, status, closed_date, jurisdiction, metadata, created_at, updated_at"
        )
        .bind(tenant_id)
        .bind(account_id)
        .bind(instrument_id)
        .bind(acquire_date)
        .bind(acquire_price)
        .bind(quantity)
        .bind(cost_basis_total)
        .bind(cost_basis_per_unit)
        .bind(cost_method)
        .bind(jurisdiction)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_err)?;
        Ok(row.into())
    }

    async fn load_open_lots(
        &self, tenant_id: Uuid, account_id: Uuid, instrument_id: &str, method: &str,
    ) -> Result<Vec<TaxLotRow>, FbError> {
        let order = match method {
            "LIFO" => "acquire_date DESC",
            "HIFO" => "cost_basis_per_unit DESC",
            "average" => "acquire_date ASC",
            _ => "acquire_date ASC",
        };
        let query = format!(
            "SELECT id, tenant_id, account_id, instrument_id, acquire_date, acquire_price, \
             original_quantity, remaining_quantity, cost_basis_total, cost_basis_per_unit, \
             cost_method, status, closed_date, jurisdiction, metadata, created_at, updated_at \
             FROM tax_lots WHERE tenant_id = $1 AND account_id = $2 AND instrument_id = $3 \
             AND status IN ('open', 'partial') ORDER BY {}",
            order
        );
        let rows: Vec<TaxLotRow> = sqlx::query_as(&query)
            .bind(tenant_id)
            .bind(account_id)
            .bind(instrument_id)
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err)?;
        Ok(rows)
    }

    pub async fn consume_lots(
        &self, tenant_id: Uuid, account_id: Uuid, instrument_id: &str,
        sale_date: NaiveDate, sale_quantity: Decimal, sale_proceeds: Decimal,
        method: &str, transaction_id: Uuid,
    ) -> Result<Vec<LotConsumption>, FbError> {
        let mut lots = self.load_open_lots(tenant_id, account_id, instrument_id, method).await?;
        let mut remaining = sale_quantity;
        let mut consumptions = Vec::new();

        if method == "average" {
            let total_qty: Decimal = lots.iter().map(|l| l.remaining_quantity).sum();
            if total_qty == Decimal::ZERO {
                return Ok(consumptions);
            }
            let total_basis: Decimal = lots.iter().map(|l| l.cost_basis_per_unit * l.remaining_quantity).sum();
            let avg_cpu = total_basis / total_qty;
            for lot in &mut lots {
                if remaining <= Decimal::ZERO { break; }
                let consume_qty = remaining.min(lot.remaining_quantity);
                let basis_consumed = avg_cpu * consume_qty;
                let proceeds = (sale_proceeds / sale_quantity) * consume_qty;
                let gain = proceeds - basis_consumed;
                let hold_days = (sale_date - lot.acquire_date).num_days().max(0) as i32;

                let row: TaxLotConsumptionRow = sqlx::query_as(
                    "INSERT INTO tax_lot_consumptions (tenant_id, transaction_id, lot_id, \
                     quantity_consumed, cost_basis_consumed, proceeds, realized_gain, \
                     holding_period_days, tax_classification, jurisdiction) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, \
                     CASE WHEN $8 >= 365 THEN 'LONG_TERM' ELSE 'SHORT_TERM' END, \
                     (SELECT jurisdiction FROM tax_lots WHERE id = $3)) \
                     RETURNING id, tenant_id, transaction_id, lot_id, quantity_consumed, \
                     cost_basis_consumed, proceeds, realized_gain, holding_period_days, \
                     tax_classification, tax_rate_applied, jurisdiction, created_at"
                )
                .bind(tenant_id)
                .bind(transaction_id)
                .bind(lot.id)
                .bind(consume_qty)
                .bind(basis_consumed)
                .bind(proceeds)
                .bind(gain)
                .bind(hold_days)
                .fetch_one(&self.pool)
                .await
                .map_err(map_db_err)?;

                let new_remaining = lot.remaining_quantity - consume_qty;
                let new_status = if new_remaining == Decimal::ZERO { "closed" } else { "partial" };
                let new_closed = if new_remaining == Decimal::ZERO { Some(sale_date) } else { None };
                sqlx::query(
                    "UPDATE tax_lots SET remaining_quantity = $1, status = $2, closed_date = $3, \
                     updated_at = NOW() WHERE id = $4"
                )
                .bind(new_remaining)
                .bind(new_status)
                .bind(new_closed)
                .bind(lot.id)
                .execute(&self.pool)
                .await
                .map_err(map_db_err)?;

                remaining -= consume_qty;
                consumptions.push(row.into());
            }
        } else {
            for lot in &mut lots {
                if remaining <= Decimal::ZERO { break; }
                let consume_qty = remaining.min(lot.remaining_quantity);
                let basis_consumed = lot.cost_basis_per_unit * consume_qty;
                let proceeds = (sale_proceeds / sale_quantity) * consume_qty;
                let gain = proceeds - basis_consumed;
                let hold_days = (sale_date - lot.acquire_date).num_days().max(0) as i32;

                let row: TaxLotConsumptionRow = sqlx::query_as(
                    "INSERT INTO tax_lot_consumptions (tenant_id, transaction_id, lot_id, \
                     quantity_consumed, cost_basis_consumed, proceeds, realized_gain, \
                     holding_period_days, tax_classification, jurisdiction) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, \
                     CASE WHEN $8 >= 365 THEN 'LONG_TERM' ELSE 'SHORT_TERM' END, \
                     (SELECT jurisdiction FROM tax_lots WHERE id = $3)) \
                     RETURNING id, tenant_id, transaction_id, lot_id, quantity_consumed, \
                     cost_basis_consumed, proceeds, realized_gain, holding_period_days, \
                     tax_classification, tax_rate_applied, jurisdiction, created_at"
                )
                .bind(tenant_id)
                .bind(transaction_id)
                .bind(lot.id)
                .bind(consume_qty)
                .bind(basis_consumed)
                .bind(proceeds)
                .bind(gain)
                .bind(hold_days)
                .fetch_one(&self.pool)
                .await
                .map_err(map_db_err)?;

                let new_remaining = lot.remaining_quantity - consume_qty;
                let new_status = if new_remaining == Decimal::ZERO { "closed" } else { "partial" };
                let new_closed = if new_remaining == Decimal::ZERO { Some(sale_date) } else { None };
                sqlx::query(
                    "UPDATE tax_lots SET remaining_quantity = $1, status = $2, closed_date = $3, \
                     updated_at = NOW() WHERE id = $4"
                )
                .bind(new_remaining)
                .bind(new_status)
                .bind(new_closed)
                .bind(lot.id)
                .execute(&self.pool)
                .await
                .map_err(map_db_err)?;

                remaining -= consume_qty;
                consumptions.push(row.into());
            }
        }

        Ok(consumptions)
    }

    pub async fn compute_tax(
        &self, tenant_id: Uuid, jurisdiction_code: &str,
    ) -> Result<TaxComputation, FbError> {
        let j: TaxJurisdictionRow = sqlx::query_as(
            "SELECT id, tenant_id, jurisdiction_code, jurisdiction_name, short_term_rate, \
             long_term_rate, long_term_threshold_days, default_cost_method, transaction_tax_rate, \
             dividend_withholding_rate, treaty_withholding_rate, wash_sale_window_days, \
             wash_sale_enabled, status, effective_from AS effective_from_date, metadata \
             FROM tax_jurisdictions WHERE tenant_id = $1 AND jurisdiction_code = $2 AND status = 'active'"
        )
        .bind(tenant_id)
        .bind(jurisdiction_code)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err)?
        .unwrap_or_else(|| TaxJurisdictionRow {
            id: Uuid::nil(), tenant_id, jurisdiction_code: jurisdiction_code.to_string(),
            jurisdiction_name: jurisdiction_code.to_string(),
            short_term_rate: dec!(0.37), long_term_rate: dec!(0.20),
            long_term_threshold_days: 365, default_cost_method: "FIFO".to_string(),
            transaction_tax_rate: Decimal::ZERO, dividend_withholding_rate: dec!(0.30),
            treaty_withholding_rate: None, wash_sale_window_days: 30,
            wash_sale_enabled: true, status: "active".to_string(),
            effective_from_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            metadata: serde_json::Value::Object(Default::default()),
        });

        let short_term_total: Option<(Decimal, Decimal)> = sqlx::query_as(
            "SELECT COALESCE(SUM(realized_gain), 0), COALESCE(SUM(quantity_consumed), 0) \
             FROM tax_lot_consumptions \
             WHERE tenant_id = $1 AND jurisdiction = $2 AND tax_classification = 'SHORT_TERM'"
        )
        .bind(tenant_id)
        .bind(jurisdiction_code)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err)?
        .map(|(a, b): (Decimal, Decimal)| (a, b));

        let long_term_total: Option<(Decimal, Decimal)> = sqlx::query_as(
            "SELECT COALESCE(SUM(realized_gain), 0), COALESCE(SUM(quantity_consumed), 0) \
             FROM tax_lot_consumptions \
             WHERE tenant_id = $1 AND jurisdiction = $2 AND tax_classification = 'LONG_TERM'"
        )
        .bind(tenant_id)
        .bind(jurisdiction_code)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err)?
        .map(|(a, b): (Decimal, Decimal)| (a, b));

        let st_gain = short_term_total.map(|(g, _)| g).unwrap_or(Decimal::ZERO);
        let lt_gain = long_term_total.map(|(g, _)| g).unwrap_or(Decimal::ZERO);
        let st_tax = st_gain * j.short_term_rate;
        let lt_tax = lt_gain * j.long_term_rate;
        let total_gain = st_gain + lt_gain;
        let total_tax = st_tax + lt_tax;
        let effective_rate = if total_gain != Decimal::ZERO {
            (total_tax / total_gain * dec!(100)).round_dp(2)
        } else {
            Decimal::ZERO
        };

        Ok(TaxComputation {
            short_term_gain: st_gain,
            long_term_gain: lt_gain,
            total_gain,
            short_term_tax: st_tax,
            long_term_tax: lt_tax,
            total_tax,
            effective_rate,
            jurisdiction_code: jurisdiction_code.to_string(),
        })
    }
}
