use async_trait::async_trait;
use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::Deserialize;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::kernel::*;

#[derive(Debug, Clone, FromRow)]
struct FeeScheduleRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub fee_code: String,
    pub fee_name: String,
    pub fee_type: String,
    pub fee_category: String,
    pub instrument_type: Option<String>,
    pub market_code: Option<String>,
    pub counterparty: Option<String>,
    pub calc_method: String,
    pub calc_config: serde_json::Value,
    pub min_amount: Option<Decimal>,
    pub max_amount: Option<Decimal>,
    pub currency: String,
    pub priority: i32,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TierConfig {
    pub tiers: Vec<Tier>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Tier {
    #[serde(rename = "from")]
    pub from_amount: Decimal,
    pub to: Option<Decimal>,
    pub rate: Decimal,
}

fn map_db_err(e: sqlx::Error) -> FbError {
    FbError::pipeline("DB", "fee", e)
}

pub struct FeeEngine {
    pool: PgPool,
}

impl FeeEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn load_fee_schedules(
        &self, tenant_id: Uuid, instrument_type: &str,
    ) -> Result<Vec<FeeScheduleRow>, FbError> {
        let rows: Vec<FeeScheduleRow> = sqlx::query_as(
            "SELECT id, tenant_id, fee_code, fee_name, fee_type, fee_category, \
             instrument_type, market_code, counterparty, calc_method, calc_config, \
             min_amount, max_amount, currency, priority, status, metadata, created_at \
             FROM fee_schedules \
             WHERE tenant_id = $1 AND status = 'active' \
             AND (instrument_type IS NULL OR instrument_type = $2) \
             ORDER BY priority ASC"
        )
        .bind(tenant_id)
        .bind(instrument_type)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err)?;
        Ok(rows)
    }

    fn compute_fee_amount(schedule: &FeeScheduleRow, basis_amount: Decimal) -> Decimal {
        let raw = match schedule.calc_method.as_str() {
            "flat" => {
                let flat_rate: Decimal = schedule.calc_config
                    .get("rate")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(dec!(0));
                flat_rate
            }
            "percentage" => {
                let pct: Decimal = schedule.calc_config
                    .get("rate")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(dec!(0));
                basis_amount * pct / dec!(100)
            }
            "bps" => {
                let bps: Decimal = schedule.calc_config
                    .get("rate")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(dec!(0));
                basis_amount * bps / dec!(10000)
            }
            "per_unit" => {
                let per_unit: Decimal = schedule.calc_config
                    .get("rate")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(dec!(0));
                let units: Decimal = schedule.calc_config
                    .get("units")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(dec!(1));
                per_unit * units
            }
            "tiered" => {
                let tier_cfg: Option<TierConfig> =
                    serde_json::from_value(schedule.calc_config.clone()).ok();
                match tier_cfg {
                    Some(cfg) => {
                        let mut remaining = basis_amount;
                        let mut total = Decimal::ZERO;
                        for tier in &cfg.tiers {
                            if remaining <= Decimal::ZERO {
                                break;
                            }
                            let band = match &tier.to {
                                Some(hi) => (*hi - tier.from_amount).min(remaining),
                                None => remaining,
                            };
                            if band <= Decimal::ZERO {
                                continue;
                            }
                            total += band * tier.rate / dec!(100);
                            remaining -= band;
                        }
                        total
                    }
                    None => Decimal::ZERO,
                }
            }
            "contract" => {
                let rate: Decimal = schedule.calc_config
                    .get("rate")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(dec!(0));
                basis_amount * rate / dec!(100)
            }
            _ => Decimal::ZERO,
        };

        let mut amount = raw;
        if let Some(min) = schedule.min_amount {
            if amount < min { amount = min; }
        }
        if let Some(max) = schedule.max_amount {
            if amount > max { amount = max; }
        }
        amount
    }
}

#[async_trait]
impl FeeCalculator for FeeEngine {
    async fn calculate(
        &self, tenant_id: &str, instrument_type: &str, basis_amount: Decimal,
    ) -> Result<Vec<FeeItem>, FbError> {
        let tid = Uuid::parse_str(tenant_id)
            .map_err(|e| FbError::pipeline("FEE", "parse", e))?;
        let schedules = self.load_fee_schedules(tid, instrument_type).await?;
        let mut items = Vec::new();
        for s in &schedules {
            let fee_amount = Self::compute_fee_amount(s, basis_amount);
            if fee_amount > Decimal::ZERO {
                items.push(FeeItem {
                    fee_code: s.fee_code.clone(),
                    fee_category: s.fee_category.clone(),
                    fee_name: s.fee_name.clone(),
                    fee_amount,
                    currency: s.currency.clone(),
                });
            }
        }
        Ok(items)
    }

    fn to_child_transactions(&self, fees: &[FeeItem], parent: &Transaction) -> Vec<Transaction> {
        let now = Utc::now();
        fees.iter().map(|fee| {
            let mut attrs = AttributeBag::new();
            attrs.set_amount(AmountType::Fee, fee.fee_amount);
            attrs.set_string("fee_code", &fee.fee_code);
            attrs.set_string("fee_category", &fee.fee_category);
            attrs.set_string("fee_name", &fee.fee_name);
            attrs.set_string("currency", &fee.currency);
            attrs.set_string("instrument_type", &parent.instrument_type);

            Transaction {
                id: Uuid::new_v4(),
                tenant_id: parent.tenant_id,
                instrument_type: "FEE".to_string(),
                instrument_id: format!("FEE-{}", fee.fee_code),
                parent_tx_id: Some(parent.id),
                root_tx_id: parent.root_tx_id,
                link_type: Some("FEE".to_string()),
                link_depth: parent.link_depth + 1,
                input_attributes: attrs,
                derived_attributes: None,
                enricher_name: "fee_engine".to_string(),
                enricher_version: "1.0.0".to_string(),
                contract_version: None,
                status: TransactionStatus::Submitted,
                idempotency_key: None,
                error_reason: None,
                metadata: None,
                created_at: now,
                posted_at: None,
            }
        }).collect()
    }
}
