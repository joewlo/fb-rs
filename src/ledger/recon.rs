use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::kernel::*;

#[derive(Debug, Clone, FromRow)]
struct AccountBalanceRow {
    pub account_id: Uuid,
    pub debit_total: Decimal,
    pub credit_total: Decimal,
    pub net_balance: Decimal,
}

#[derive(Debug, Clone, FromRow)]
struct PositionBalanceRow {
    pub account_id: Uuid,
    pub instrument_id: String,
    pub quantity: Decimal,
    pub cost_basis_total: Decimal,
}

#[derive(Debug, Clone)]
pub struct ReconDiscrepancy {
    pub account_id: Uuid,
    pub instrument_id: Option<String>,
    pub source: String,
    pub message: String,
    pub amount_difference: Decimal,
}

#[derive(Debug, Clone)]
pub struct ReconResult {
    pub as_of_date: NaiveDate,
    pub tenant_id: Uuid,
    pub passed: bool,
    pub total_discrepancies: usize,
    pub total_difference: Decimal,
    pub discrepancies: Vec<ReconDiscrepancy>,
}

fn map_db_err(e: sqlx::Error) -> FbError {
    FbError::pipeline("DB", "recon", e)
}

pub struct ReconEngine {
    pool: PgPool,
}

impl ReconEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn reconcile(
        &self, tenant_id: Uuid, as_of_date: NaiveDate,
    ) -> Result<ReconResult, FbError> {
        let mut discrepancies = Vec::new();

        let account_balances: Vec<AccountBalanceRow> = sqlx::query_as(
            "SELECT account_id, \
             COALESCE(SUM(CASE WHEN side = 'DEBIT' THEN amount ELSE 0 END), 0) AS debit_total, \
             COALESCE(SUM(CASE WHEN side = 'CREDIT' THEN amount ELSE 0 END), 0) AS credit_total, \
             COALESCE(SUM(CASE WHEN side = 'DEBIT' THEN amount ELSE -amount END), 0) AS net_balance \
             FROM journal_entries \
             WHERE tenant_id = $1 AND value_date <= $2 \
             GROUP BY account_id"
        )
        .bind(tenant_id)
        .bind(as_of_date)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err)?;

        let positions: Vec<PositionBalanceRow> = sqlx::query_as(
            "SELECT account_id, instrument_id, quantity, cost_basis_total \
             FROM position_cost_basis \
             WHERE tenant_id = $1 AND quantity <> 0"
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err)?;

        let je_accounts: std::collections::HashSet<Uuid> = account_balances
            .iter().map(|a| a.account_id).collect();
        let pos_accounts: std::collections::HashSet<Uuid> = positions
            .iter().map(|p| p.account_id).collect();

        for acct in je_accounts.difference(&pos_accounts) {
            let bal = account_balances.iter().find(|a| a.account_id == *acct).unwrap();
            discrepancies.push(ReconDiscrepancy {
                account_id: *acct,
                instrument_id: None,
                source: "journal_entries".to_string(),
                message: format!(
                    "Account has journal entries (net: {}) but no position records", bal.net_balance
                ),
                amount_difference: bal.net_balance,
            });
        }

        for acct in pos_accounts.difference(&je_accounts) {
            let pos = positions.iter().find(|p| p.account_id == *acct).unwrap();
            discrepancies.push(ReconDiscrepancy {
                account_id: *acct,
                instrument_id: Some(pos.instrument_id.clone()),
                source: "position_cost_basis".to_string(),
                message: "Account has position records but no journal entries".to_string(),
                amount_difference: pos.cost_basis_total,
            });
        }

        for acct in je_accounts.intersection(&pos_accounts) {
            let j_entries: Vec<AccountBalanceRow> = account_balances
                .iter().filter(|a| a.account_id == *acct).cloned().collect();
            let p_entries: Vec<PositionBalanceRow> = positions
                .iter().filter(|p| p.account_id == *acct).cloned().collect();

            for je in &j_entries {
                let position_sum: Decimal = p_entries.iter()
                    .map(|p| p.cost_basis_total).sum();

                let tolerance = Decimal::new(1, 2);
                let diff = je.net_balance - position_sum;
                if diff.abs() > tolerance {
                    discrepancies.push(ReconDiscrepancy {
                        account_id: *acct,
                        instrument_id: None,
                        source: "balance_mismatch".to_string(),
                        message: format!(
                            "Account will have JE balance {} vs position cost {}", 
                            je.net_balance, position_sum
                        ),
                        amount_difference: diff,
                    });
                }
            }

            for je in &j_entries {
                if je.debit_total != je.credit_total && je.debit_total > Decimal::ZERO
                    && je.credit_total > Decimal::ZERO
                {
                    let unbalanced_diff = je.debit_total - je.credit_total;
                    if unbalanced_diff.abs() > Decimal::new(1, 2) {
                        discrepancies.push(ReconDiscrepancy {
                            account_id: *acct,
                            instrument_id: None,
                            source: "unbalanced".to_string(),
                            message: format!(
                                "Account has unbalanced journal entries: D={} C={} diff={}",
                                je.debit_total, je.credit_total, unbalanced_diff,
                            ),
                            amount_difference: unbalanced_diff,
                        });
                    }
                }
            }
        }

        let total_diff: Decimal = discrepancies.iter()
            .map(|d| d.amount_difference.abs()).sum();

        Ok(ReconResult {
            as_of_date,
            tenant_id,
            passed: discrepancies.is_empty(),
            total_discrepancies: discrepancies.len(),
            total_difference: total_diff,
            discrepancies,
        })
    }
}
