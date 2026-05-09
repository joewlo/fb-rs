use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::kernel::*;

#[derive(Debug, Clone, FromRow)]
struct CashFlowRow {
    pub value_date: NaiveDate,
    pub net_flow: Decimal,
}

#[derive(Debug, Clone)]
pub struct CashFlow {
    pub date: NaiveDate,
    pub amount: Decimal,
}

#[derive(Debug, Clone)]
pub struct PerformanceResult {
    pub irr: Option<Decimal>,
    pub modified_dietz: Option<Decimal>,
    pub twr: Option<Decimal>,
    pub start_value: Decimal,
    pub end_value: Decimal,
    pub net_cash_flow: Decimal,
    pub total_return: Decimal,
}

fn map_db_err(e: sqlx::Error) -> FbError {
    FbError::pipeline("DB", "perf", e)
}

pub struct PerformanceEngine {
    pool: PgPool,
}

impl PerformanceEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn load_cash_flows(
        &self, tenant_id: Uuid, account_id: Uuid,
        start_date: NaiveDate, end_date: NaiveDate,
    ) -> Result<Vec<CashFlowRow>, FbError> {
        let rows: Vec<CashFlowRow> = sqlx::query_as(
            "SELECT value_date, SUM(CASE WHEN side = 'DEBIT' THEN amount ELSE -amount END) AS net_flow \
             FROM journal_entries \
             WHERE tenant_id = $1 AND account_id = $2 \
             AND value_date >= $3 AND value_date <= $4 \
             GROUP BY value_date ORDER BY value_date ASC"
        )
        .bind(tenant_id)
        .bind(account_id)
        .bind(start_date)
        .bind(end_date)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err)?;
        Ok(rows)
    }

    async fn load_position_market_values(
        &self, tenant_id: Uuid, account_id: Uuid, dates: Vec<NaiveDate>,
    ) -> Result<std::collections::HashMap<NaiveDate, Decimal>, FbError> {
        let mut map = std::collections::HashMap::new();
        for date in dates {
            let mv: Option<(Decimal,)> = sqlx::query_as(
                "SELECT COALESCE(SUM(CASE WHEN quantity > 0 THEN quantity * COALESCE(last_price, 0) \
                 ELSE 0 END), 0) FROM position_cost_basis \
                 WHERE tenant_id = $1 AND account_id = $2"
            )
            .bind(tenant_id)
            .bind(account_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_err)?;
            map.insert(date, mv.map(|(v,)| v).unwrap_or(Decimal::ZERO));
        }
        Ok(map)
    }

    async fn load_position_valuation(
        &self, tenant_id: Uuid, account_id: Uuid,
    ) -> Result<Decimal, FbError> {
        let val: Option<(Decimal,)> = sqlx::query_as(
            "SELECT COALESCE(SUM(quantity * cost_basis_total / NULLIF(quantity, 0)), 0) \
             FROM position_cost_basis WHERE tenant_id = $1 AND account_id = $2 AND quantity > 0"
        )
        .bind(tenant_id)
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err)?;
        Ok(val.map(|(v,)| v).unwrap_or(Decimal::ZERO))
    }

    pub async fn calculate_performance(
        &self, tenant_id: Uuid, account_id: Uuid,
        start_date: NaiveDate, end_date: NaiveDate,
    ) -> Result<PerformanceResult, FbError> {
        let flows = self.load_cash_flows(tenant_id, account_id, start_date, end_date).await?;
        let start_value = self.load_position_valuation(tenant_id, account_id).await?;
        let end_value = self.load_position_valuation(tenant_id, account_id).await?;

        let cash: Vec<CashFlow> = flows.iter().map(|r| CashFlow {
            date: r.value_date,
            amount: r.net_flow,
        }).collect();

        let net_cash_flow: Decimal = cash.iter().map(|c| c.amount).sum();
        let total_return = if start_value != Decimal::ZERO {
            (end_value - start_value - net_cash_flow) / start_value
        } else {
            Decimal::ZERO
        };

        let irr = Self::compute_irr(&cash, start_date, start_value, end_date, end_value);
        let modified_dietz = Self::compute_modified_dietz(
            &cash, start_date, end_date, start_value, end_value,
        );
        let twr = Self::compute_twr(&cash, start_date, end_date, start_value, end_value);

        Ok(PerformanceResult {
            irr,
            modified_dietz,
            twr,
            start_value,
            end_value,
            net_cash_flow,
            total_return,
        })
    }

    fn compute_irr(
        cash_flows: &[CashFlow], start_date: NaiveDate, start_value: Decimal,
        end_date: NaiveDate, end_value: Decimal,
    ) -> Option<Decimal> {
        let total_days = (end_date - start_date).num_days() as f64;
        if total_days <= 0.0 {
            return None;
        }

        let mut all_flows: Vec<(f64, f64)> = Vec::new();
        all_flows.push((0.0, -start_value.to_f64().unwrap_or(0.0)));
        for cf in cash_flows {
            let days = (cf.date - start_date).num_days() as f64;
            let amt = cf.amount.to_f64().unwrap_or(0.0);
            if amt.abs() > 1e-12 {
                all_flows.push((days / 365.25, -amt));
            }
        }
        all_flows.push((total_days / 365.25, end_value.to_f64().unwrap_or(0.0)));

        let mut guess: f64 = 0.1;
        for _ in 0..200 {
            let (pv, dpv) = all_flows.iter().fold((0.0_f64, 0.0_f64), |(s, ds), &(t, cf)| {
                let factor = (1.0 + guess).powf(-t);
                let dfactor = -t * (1.0 + guess).powf(-t - 1.0);
                (s + cf * factor, ds + cf * dfactor)
            });
            if dpv.abs() < 1e-12 { break; }
            let new_guess = guess - pv / dpv;
            if (new_guess - guess).abs() < 1e-9 { break; }
            guess = new_guess;
            if guess < -0.99 { guess = -0.50; }
            if guess > 10.0 { guess = 5.0; }
        }

        let irr = Decimal::from_f64_retain(guess).unwrap_or(Decimal::ZERO);
        Some(irr)
    }

    fn compute_modified_dietz(
        cash_flows: &[CashFlow], start_date: NaiveDate, end_date: NaiveDate,
        start_value: Decimal, end_value: Decimal,
    ) -> Option<Decimal> {
        let total_days = (end_date - start_date).num_days() as f64;
        if total_days <= 0.0 || start_value.is_zero() {
            return None;
        }

        let weighted_flows: f64 = cash_flows.iter().map(|cf| {
            let days_remaining = (end_date - cf.date).num_days() as f64;
            let weight = days_remaining / total_days;
            cf.amount.to_f64().unwrap_or(0.0) * weight
        }).sum();

        let sv = start_value.to_f64().unwrap_or(0.0);
        let ev = end_value.to_f64().unwrap_or(0.0);
        let ncf: f64 = cash_flows.iter().map(|cf| cf.amount.to_f64().unwrap_or(0.0)).sum();

        let ret = (ev - sv - ncf) / (sv + weighted_flows);
        Some(Decimal::from_f64_retain(ret).unwrap_or(Decimal::ZERO))
    }

    fn compute_twr(
        cash_flows: &[CashFlow], _start_date: NaiveDate, _end_date: NaiveDate,
        start_value: Decimal, end_value: Decimal,
    ) -> Option<Decimal> {
        if cash_flows.is_empty() {
            if start_value.is_zero() {
                return None;
            }
            return Some((end_value - start_value) / start_value);
        }

        let sv = start_value.to_f64().unwrap_or(0.0);
        let _ev = end_value.to_f64().unwrap_or(0.0);

        let mut sub_periods: Vec<(f64, f64, f64)> = Vec::new();
        let mut prev_val = sv;

        for cf in cash_flows {
            let cf_val = cf.amount.to_f64().unwrap_or(0.0);
            let sub_ret = if prev_val.abs() > 1e-12 {
                (cf_val - prev_val) / prev_val
            } else {
                0.0
            };
            sub_periods.push((prev_val, cf_val, sub_ret));
            prev_val = cf_val;
        }

        if sub_periods.is_empty() {
            return None;
        }

        let product: f64 = sub_periods.iter().fold(1.0, |acc, (_, _, r)| acc * (1.0 + r));
        let twr = product - 1.0;
        Some(Decimal::from_f64_retain(twr).unwrap_or(Decimal::ZERO))
    }
}
