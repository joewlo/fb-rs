use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::kernel::*;

#[derive(Debug, Clone, FromRow)]
struct PositionStatementRow {
    pub instrument_id: String,
    pub quantity: Decimal,
    pub cost_basis_per_unit: Option<Decimal>,
    pub cost_basis_total: Decimal,
    pub last_price: Option<Decimal>,
    pub total_realized_pnl: Decimal,
    pub total_unrealized_pnl: Decimal,
}

#[derive(Debug, Clone)]
pub struct PositionStatement {
    pub as_of_date: NaiveDate,
    pub tenant_id: Uuid,
    pub account_id: Uuid,
    pub positions: Vec<PositionStatementEntry>,
    pub total_market_value: Decimal,
    pub total_cost_basis: Decimal,
    pub total_unrealized_pnl: Decimal,
    pub total_realized_pnl: Decimal,
}

#[derive(Debug, Clone)]
pub struct PositionStatementEntry {
    pub instrument_id: String,
    pub quantity: Decimal,
    pub cost_basis_per_unit: Option<Decimal>,
    pub cost_basis_total: Decimal,
    pub market_price: Option<Decimal>,
    pub market_value: Decimal,
    pub unrealized_pnl: Decimal,
    pub realized_pnl: Decimal,
}

#[derive(Debug, Clone)]
pub struct TransactionStatement {
    pub from_date: NaiveDate,
    pub to_date: NaiveDate,
    pub tenant_id: Uuid,
    pub account_id: Uuid,
    pub entries: Vec<TransactionStatementEntry>,
    pub total_debits: Decimal,
    pub total_credits: Decimal,
    pub net_amount: Decimal,
}

#[derive(Debug, Clone, FromRow)]
pub struct TransactionStatementEntry {
    pub transaction_id: Uuid,
    pub entry_sequence: i32,
    pub value_date: NaiveDate,
    pub side: String,
    pub amount: Decimal,
    pub currency: String,
    pub narrative: String,
    pub instrument_type: String,
    pub instrument_id: String,
}

fn map_db_err(e: sqlx::Error) -> FbError {
    FbError::pipeline("DB", "stmt", e)
}

pub struct StatementsEngine {
    pool: PgPool,
}

impl StatementsEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn generate_position_statement(
        &self, tenant_id: Uuid, account_id: Uuid, as_of_date: NaiveDate,
    ) -> Result<PositionStatement, FbError> {
        let rows: Vec<PositionStatementRow> = sqlx::query_as(
            "SELECT instrument_id, quantity, cost_basis_per_unit, cost_basis_total, \
             last_price, total_realized_pnl, total_unrealized_pnl \
             FROM position_cost_basis \
             WHERE tenant_id = $1 AND account_id = $2"
        )
        .bind(tenant_id)
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err)?;

        let mut entries = Vec::new();
        let mut total_mv = Decimal::ZERO;
        let mut total_cost = Decimal::ZERO;
        let mut total_unrealized = Decimal::ZERO;
        let mut total_realized = Decimal::ZERO;

        for row in &rows {
            let price = row.last_price.unwrap_or(Decimal::ZERO);
            let mv = price * row.quantity;
            let realized = row.total_realized_pnl;
            let unrealized = row.total_unrealized_pnl;

            entries.push(PositionStatementEntry {
                instrument_id: row.instrument_id.clone(),
                quantity: row.quantity,
                cost_basis_per_unit: row.cost_basis_per_unit,
                cost_basis_total: row.cost_basis_total,
                market_price: row.last_price,
                market_value: mv,
                unrealized_pnl: unrealized,
                realized_pnl: realized,
            });

            total_mv += mv;
            total_cost += row.cost_basis_total;
            total_unrealized += unrealized;
            total_realized += realized;
        }

        Ok(PositionStatement {
            as_of_date,
            tenant_id,
            account_id,
            positions: entries,
            total_market_value: total_mv,
            total_cost_basis: total_cost,
            total_unrealized_pnl: total_unrealized,
            total_realized_pnl: total_realized,
        })
    }

    pub async fn generate_transaction_statement(
        &self, tenant_id: Uuid, account_id: Uuid,
        from_date: NaiveDate, to_date: NaiveDate,
    ) -> Result<TransactionStatement, FbError> {
        let entries: Vec<TransactionStatementEntry> = sqlx::query_as(
            "SELECT je.transaction_id, je.entry_sequence, je.value_date, je.side, \
             je.amount, je.currency, je.narrative, \
             COALESCE(t.instrument_type, 'UNKNOWN') AS instrument_type, \
             COALESCE(t.instrument_id, 'UNKNOWN') AS instrument_id \
             FROM journal_entries je \
             LEFT JOIN transactions t ON je.transaction_id = t.id AND je.tenant_id = t.tenant_id \
             WHERE je.tenant_id = $1 AND je.account_id = $2 \
             AND je.value_date >= $3 AND je.value_date <= $4 \
             ORDER BY je.value_date ASC, je.entry_sequence ASC"
        )
        .bind(tenant_id)
        .bind(account_id)
        .bind(from_date)
        .bind(to_date)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err)?;

        let total_debits: Decimal = entries.iter()
            .filter(|e| e.side == "DEBIT")
            .map(|e| e.amount)
            .sum();

        let total_credits: Decimal = entries.iter()
            .filter(|e| e.side == "CREDIT")
            .map(|e| e.amount)
            .sum();

        let net_amount = total_debits - total_credits;

        Ok(TransactionStatement {
            from_date,
            to_date,
            tenant_id,
            account_id,
            entries,
            total_debits,
            total_credits,
            net_amount,
        })
    }
}
