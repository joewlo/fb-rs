use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::kernel::*;

#[derive(Debug, Clone, FromRow)]
struct PositionCostBasisRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub account_id: Uuid,
    pub instrument_id: String,
    pub quantity: Decimal,
    pub cost_basis_total: Decimal,
    pub cost_basis_per_unit: Option<Decimal>,
    pub total_realized_pnl: Decimal,
    pub total_unrealized_pnl: Decimal,
    pub last_price: Option<Decimal>,
    pub version: i64,
    pub updated_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
struct PriceFeedRow {
    pub id: i64,
    pub tenant_id: Uuid,
    pub instrument_id: String,
    pub price_type: String,
    pub price: Decimal,
    pub bid: Option<Decimal>,
    pub ask: Option<Decimal>,
    pub volume_24h: Option<Decimal>,
    pub source: String,
    pub timestamp: chrono::DateTime<Utc>,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct PnlSnapshotRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub desk: String,
    pub instrument_id: Option<String>,
    pub snapshot_date: NaiveDate,
    pub realized_pnl: Decimal,
    pub unrealized_pnl: Decimal,
    pub commission_paid: Decimal,
    pub fee_paid: Decimal,
    pub interest_accrued: Decimal,
    pub gross_pnl: Decimal,
    pub net_pnl: Decimal,
    pub trade_count: i32,
    pub updated_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct PositionSummary {
    pub instrument_id: String,
    pub quantity: Decimal,
    pub cost_basis_total: Decimal,
    pub cost_basis_per_unit: Option<Decimal>,
    pub total_realized_pnl: Decimal,
    pub total_unrealized_pnl: Decimal,
    pub last_price: Option<Decimal>,
    pub market_value: Decimal,
}

fn map_db_err(e: sqlx::Error) -> FbError {
    FbError::pipeline("DB", "pnl", e)
}

pub struct PNLEngine {
    pool: PgPool,
}

impl PNLEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_position_summary(
        &self, tenant_id: Uuid, account_id: Uuid,
    ) -> Result<Vec<PositionSummary>, FbError> {
        let rows: Vec<PositionCostBasisRow> = sqlx::query_as(
            "SELECT id, tenant_id, account_id, instrument_id, quantity, cost_basis_total, \
             cost_basis_per_unit, total_realized_pnl, total_unrealized_pnl, last_price, version, \
             updated_at FROM position_cost_basis \
             WHERE tenant_id = $1 AND account_id = $2 AND quantity <> 0"
        )
        .bind(tenant_id)
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err)?;

        Ok(rows.into_iter().map(|r| PositionSummary {
            instrument_id: r.instrument_id,
            quantity: r.quantity,
            cost_basis_total: r.cost_basis_total,
            cost_basis_per_unit: r.cost_basis_per_unit,
            total_realized_pnl: r.total_realized_pnl,
            total_unrealized_pnl: r.total_unrealized_pnl,
            last_price: r.last_price,
            market_value: r.last_price.unwrap_or(Decimal::ZERO) * r.quantity,
        }).collect())
    }

    pub async fn calculate_daily_pnl(
        &self, tenant_id: Uuid, desk: &str, date: NaiveDate,
    ) -> Result<PnlSnapshotRow, FbError> {
        let row: PnlSnapshotRow = sqlx::query_as(
            "SELECT id, tenant_id, desk, instrument_id, snapshot_date, realized_pnl, \
             unrealized_pnl, commission_paid, fee_paid, interest_accrued, gross_pnl, \
             net_pnl, trade_count, updated_at FROM pnl_snapshots \
             WHERE tenant_id = $1 AND desk = $2 AND snapshot_date = $3 AND instrument_id IS NULL"
        )
        .bind(tenant_id)
        .bind(desk)
        .bind(date)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err)?
        .unwrap_or_else(|| PnlSnapshotRow {
            id: Uuid::nil(),
            tenant_id,
            desk: desk.to_string(),
            instrument_id: None,
            snapshot_date: date,
            realized_pnl: Decimal::ZERO,
            unrealized_pnl: Decimal::ZERO,
            commission_paid: Decimal::ZERO,
            fee_paid: Decimal::ZERO,
            interest_accrued: Decimal::ZERO,
            gross_pnl: Decimal::ZERO,
            net_pnl: Decimal::ZERO,
            trade_count: 0,
            updated_at: Utc::now(),
        });
        Ok(row)
    }

    pub async fn ingest_price(
        &self, tenant_id: Uuid, instrument_id: &str, price_type: &str,
        price: Decimal, bid: Option<Decimal>, ask: Option<Decimal>,
        volume_24h: Option<Decimal>, source: &str,
    ) -> Result<(), FbError> {
        sqlx::query(
            "INSERT INTO price_feed (tenant_id, instrument_id, price_type, price, bid, ask, \
             volume_24h, source) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
        )
        .bind(tenant_id)
        .bind(instrument_id)
        .bind(price_type)
        .bind(price)
        .bind(bid)
        .bind(ask)
        .bind(volume_24h)
        .bind(source)
        .execute(&self.pool)
        .await
        .map_err(map_db_err)?;

        self.mark_to_market(tenant_id, instrument_id, price).await
    }

    async fn mark_to_market(
        &self, tenant_id: Uuid, instrument_id: &str, price: Decimal,
    ) -> Result<(), FbError> {
        let positions: Vec<PositionCostBasisRow> = sqlx::query_as(
            "SELECT id, tenant_id, account_id, instrument_id, quantity, cost_basis_total, \
             cost_basis_per_unit, total_realized_pnl, total_unrealized_pnl, last_price, version, \
             updated_at FROM position_cost_basis \
             WHERE tenant_id = $1 AND instrument_id = $2 AND quantity <> 0"
        )
        .bind(tenant_id)
        .bind(instrument_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err)?;

        for pos in positions {
            let spu = match pos.cost_basis_per_unit {
                Some(cpu) if cpu != Decimal::ZERO => cpu,
                _ => continue,
            };
            let unrealized = (price - spu) * pos.quantity;
            let version = pos.version + 1;
            sqlx::query(
                "UPDATE position_cost_basis SET last_price = $1, total_unrealized_pnl = $2, \
                 version = $3, updated_at = NOW() WHERE id = $4 AND version = $5"
            )
            .bind(price)
            .bind(unrealized)
            .bind(version)
            .bind(pos.id)
            .bind(pos.version)
            .execute(&self.pool)
            .await
            .map_err(map_db_err)?;
        }
        Ok(())
    }
}

#[async_trait]
impl PositionTracker for PNLEngine {
    async fn track_position(
        &self, tenant_id: Uuid, account_id: Uuid, instrument_id: &str,
        side: &str, quantity: Decimal, amount: Decimal,
    ) -> Result<(), FbError> {
        let is_buy = side.eq_ignore_ascii_case("BUY")
            || side.eq_ignore_ascii_case("DEBIT");
        let qty_delta = if is_buy { quantity } else { -quantity };
        let cost_delta = if is_buy { amount } else { -amount };

        let existing: Option<PositionCostBasisRow> = sqlx::query_as(
            "SELECT id, tenant_id, account_id, instrument_id, quantity, cost_basis_total, \
             cost_basis_per_unit, total_realized_pnl, total_unrealized_pnl, last_price, version, \
             updated_at FROM position_cost_basis \
             WHERE tenant_id = $1 AND account_id = $2 AND instrument_id = $3"
        )
        .bind(tenant_id)
        .bind(account_id)
        .bind(instrument_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err)?;

        if let Some(curr) = existing {
            let new_qty = curr.quantity + qty_delta;
            let new_cost = curr.cost_basis_total + cost_delta;
            let new_cpu = if new_qty != Decimal::ZERO {
                Some(new_cost / new_qty)
            } else {
                None
            };

            let mut realized = curr.total_realized_pnl;
            if !is_buy && curr.quantity > Decimal::ZERO {
                let sell_qty = quantity.min(curr.quantity);
                let sell_basis = curr.cost_basis_per_unit
                    .unwrap_or(Decimal::ZERO) * sell_qty;
                realized += amount - sell_basis;
            }

            let price = curr.last_price.unwrap_or(Decimal::ZERO);
            let unrealized = if new_qty != Decimal::ZERO {
                let cpu = new_cpu.unwrap_or(Decimal::ZERO);
                if cpu != Decimal::ZERO { (price - cpu) * new_qty } else { Decimal::ZERO }
            } else {
                Decimal::ZERO
            };

            let version = curr.version + 1;
            sqlx::query(
                "UPDATE position_cost_basis SET quantity = $1, cost_basis_total = $2, \
                 cost_basis_per_unit = $3, total_realized_pnl = $4, total_unrealized_pnl = $5, \
                 version = $6, updated_at = NOW() WHERE id = $7 AND version = $8"
            )
            .bind(new_qty)
            .bind(new_cost)
            .bind(new_cpu)
            .bind(realized)
            .bind(unrealized)
            .bind(version)
            .bind(curr.id)
            .bind(curr.version)
            .execute(&self.pool)
            .await
            .map_err(map_db_err)?;
        } else if is_buy {
            let cpu = if quantity != Decimal::ZERO {
                Some(amount / quantity)
            } else {
                None
            };
            sqlx::query(
                "INSERT INTO position_cost_basis (tenant_id, account_id, instrument_id, quantity, \
                 cost_basis_total, cost_basis_per_unit, total_realized_pnl, total_unrealized_pnl, \
                 last_price, version) VALUES ($1, $2, $3, $4, $5, $6, 0, 0, 0, 0)"
            )
            .bind(tenant_id)
            .bind(account_id)
            .bind(instrument_id)
            .bind(quantity)
            .bind(amount)
            .bind(cpu)
            .execute(&self.pool)
            .await
            .map_err(map_db_err)?;
        }

        Ok(())
    }
}
