use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::kernel::*;

#[derive(Debug, Clone, FromRow)]
struct SanctionedEntityRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub list_id: Uuid,
    pub entity_type: String,
    pub full_name: String,
    pub aliases: serde_json::Value,
    pub identifiers: serde_json::Value,
    pub sanctions_program: Option<String>,
    pub date_listed: Option<NaiveDate>,
    pub risk_level: String,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
struct AmlRuleRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub rule_code: String,
    pub rule_name: String,
    pub rule_type: String,
    pub params: serde_json::Value,
    pub instrument_type: Option<String>,
    pub jurisdiction: Option<String>,
    pub action: String,
    pub severity: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
struct ComplianceAlertRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub transaction_id: Option<Uuid>,
    pub alert_type: String,
    pub alert_severity: String,
    pub alert_message: String,
    pub matched_entity: Option<String>,
    pub match_score: Option<Decimal>,
    pub source: Option<String>,
    pub status: String,
    pub resolved_by: Option<String>,
    pub resolved_at: Option<chrono::DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<Utc>,
}

fn map_db_err(e: sqlx::Error) -> FbError {
    FbError::pipeline("DB", "compliance", e)
}

fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.to_lowercase().chars().collect();
    let b_chars: Vec<char> = b.to_lowercase().chars().collect();
    let alen = a_chars.len();
    let blen = b_chars.len();
    if alen == 0 { return blen; }
    if blen == 0 { return alen; }

    let mut prev: Vec<usize> = (0..=blen).collect();
    let mut curr = vec![0usize; blen + 1];

    for i in 1..=alen {
        curr[0] = i;
        for j in 1..=blen {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[blen]
}

fn fuzzy_score(query: &str, target: &str) -> f64 {
    let dist = levenshtein_distance(query, target);
    let max_len = query.len().max(target.len()).max(1) as f64;
    1.0 - (dist as f64 / max_len)
}

pub struct ComplianceEngine {
    pool: PgPool,
}

impl ComplianceEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn seed_sanctions(&self, tenant_id: Uuid) -> Result<(), FbError> {
        let list_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO sanctions_lists (id, tenant_id, list_name, list_authority, list_version, \
             entity_count) VALUES ($1, $2, 'OFAC SDN', 'US OFAC', '2024.1', 3) \
             ON CONFLICT (tenant_id, list_name, list_version) DO NOTHING"
        )
        .bind(list_id)
        .bind(tenant_id)
        .execute(&self.pool)
        .await
        .map_err(map_db_err)?;

        let entities = vec![
            ("VIKTOR BOUT", "INDIVIDUAL", "SDGT", "HIGH"),
            ("NORTH KOREAN TRADE BANK", "ENTITY", "DPRK", "CRITICAL"),
            ("AL-QAEDA SANCTIONS NETWORK", "ENTITY", "SDGT", "HIGH"),
            ("CARTEL DE JALISCO NUEVA GENERACION", "ENTITY", "SDNTK", "HIGH"),
            ("ISLAMIC REVOLUTIONARY GUARD CORPS", "ENTITY", "IRAN", "HIGH"),
            ("EVGENY PRIGOZHIN", "INDIVIDUAL", "RUSSIA", "HIGH"),
            ("SULEIMAN ABU GHAITH", "INDIVIDUAL", "SDGT", "MEDIUM"),
            ("HIZBALLAH INTERNATIONAL", "ENTITY", "SDGT", "HIGH"),
            ("MAUTE GROUP", "ENTITY", "SDGT", "MEDIUM"),
            ("ABU SAYYAF GROUP", "ENTITY", "SDGT", "HIGH"),
        ];

        for (name, entity_type, program, risk) in &entities {
            sqlx::query(
                "INSERT INTO sanctioned_entities (tenant_id, list_id, entity_type, full_name, \
                 sanctions_program, risk_level) VALUES ($1, $2, $3, $4, $5, $6) \
                 ON CONFLICT DO NOTHING"
            )
            .bind(tenant_id)
            .bind(list_id)
            .bind(entity_type)
            .bind(name)
            .bind(program)
            .bind(risk)
            .execute(&self.pool)
            .await
            .map_err(map_db_err)?;
        }

        sqlx::query(
            "INSERT INTO aml_rules (tenant_id, rule_code, rule_name, rule_type, params, \
             action, severity) VALUES \
             ($1, 'AML_VEL_01', 'Daily Volume Threshold', 'VOLUME', \
             '{\"max_daily_volume\": 1000000, \"currency\": \"USD\"}', 'ALERT', 'MEDIUM'), \
             ($1, 'AML_VEL_02', 'Structuring Detection', 'STRUCTURING', \
             '{\"max_single_amount\": 10000, \"window_days\": 30}', 'BLOCK', 'HIGH'), \
             ($1, 'AML_GEO_01', 'Restricted Jurisdiction', 'GEO_RESTRICTION', \
             '{\"restricted_countries\": [\"KP\", \"IR\", \"SY\", \"CU\"]}', 'BLOCK', 'CRITICAL'), \
             ($1, 'AML_CP_01', 'Counterparty Concentration', 'COUNTERPARTY_CONCENTRATION', \
             '{\"max_pct_single_cp\": 30}', 'FLAG', 'LOW') \
             ON CONFLICT (tenant_id, rule_code) DO NOTHING"
        )
        .bind(tenant_id)
        .execute(&self.pool)
        .await
        .map_err(map_db_err)?;

        Ok(())
    }

    async fn sanctions_check(
        &self, tenant_id: Uuid, counterparty: &str, _instrument_type: &str,
        _amount: Decimal, _trade_date: NaiveDate,
    ) -> Result<Vec<ComplianceAlert>, FbError> {
        let entities: Vec<SanctionedEntityRow> = sqlx::query_as(
            "SELECT id, tenant_id, list_id, entity_type, full_name, aliases, identifiers, \
             sanctions_program, date_listed, risk_level, metadata, created_at \
             FROM sanctioned_entities WHERE tenant_id = $1"
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err)?;

        let mut alerts = Vec::new();
        let threshold = 0.75;
        for entity in &entities {
            let name_score = fuzzy_score(counterparty, &entity.full_name);
            if name_score >= threshold {
                alerts.push(ComplianceAlert {
                    alert_type: "SANCTIONS_MATCH".to_string(),
                    severity: entity.risk_level.to_lowercase(),
                    message: format!(
                        "Counterparty '{}' matched sanctioned entity '{}' (score: {:.2}%, program: {:?})",
                        counterparty, entity.full_name, name_score * 100.0, entity.sanctions_program,
                    ),
                });
            }

            if let Ok(aliases) = serde_json::from_value::<Vec<String>>(entity.aliases.clone()) {
                for alias in &aliases {
                    let alias_score = fuzzy_score(counterparty, alias);
                    if alias_score >= threshold {
                        alerts.push(ComplianceAlert {
                            alert_type: "SANCTIONS_MATCH".to_string(),
                            severity: entity.risk_level.to_lowercase(),
                            message: format!(
                                "Counterparty '{}' matched alias '{}' of sanctioned entity '{}' (score: {:.2}%)",
                                counterparty, alias, entity.full_name, alias_score * 100.0,
                            ),
                        });
                    }
                }
            }
        }

        Ok(alerts)
    }

    async fn aml_check(
        &self, tenant_id: Uuid, _counterparty: &str, instrument_type: &str,
        amount: Decimal, _trade_date: NaiveDate,
    ) -> Result<Vec<ComplianceAlert>, FbError> {
        let rules: Vec<AmlRuleRow> = sqlx::query_as(
            "SELECT id, tenant_id, rule_code, rule_name, rule_type, params, instrument_type, \
             jurisdiction, action, severity, status, metadata, created_at \
             FROM aml_rules WHERE tenant_id = $1 AND status = 'active' \
             AND (instrument_type IS NULL OR instrument_type = $2)"
        )
        .bind(tenant_id)
        .bind(instrument_type)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err)?;

        let mut alerts = Vec::new();

        for rule in &rules {
            match rule.rule_type.as_str() {
                "VOLUME" => {
                    let max_daily: Option<f64> = rule.params
                        .get("max_daily_volume")
                        .and_then(|v| v.as_f64());
                    if let Some(max) = max_daily {
                        let amt = amount.to_f64().unwrap_or(0.0);
                        if amt > max {
                            alerts.push(ComplianceAlert {
                                alert_type: "AML_VELOCITY".to_string(),
                                severity: rule.severity.clone(),
                                message: format!(
                                    "Daily volume {} exceeds threshold {} for rule {}",
                                    amt, max, rule.rule_name,
                                ),
                            });
                        }
                    }
                }
                "STRUCTURING" => {
                    let max_amount: Option<f64> = rule.params
                        .get("max_single_amount")
                        .and_then(|v| v.as_f64());
                    if let Some(max) = max_amount {
                        let amt = amount.to_f64().unwrap_or(0.0);
                        if amt > 0.0 && amt < max && amt > max * 0.8 {
                            alerts.push(ComplianceAlert {
                                alert_type: "AML_STRUCTURING".to_string(),
                                severity: rule.severity.clone(),
                                message: format!(
                                    "Transaction amount {} appears to be structured (below {}) for rule {}",
                                    amt, max, rule.rule_name,
                                ),
                            });
                        }
                    }
                }
                "GEO_RESTRICTION" => {
                    if let Some(_countries) = rule.params.get("restricted_countries") {
                        alerts.push(ComplianceAlert {
                            alert_type: "GEO_RESTRICTION".to_string(),
                            severity: rule.severity.clone(),
                            message: format!(
                                "Geographic restriction rule '{}' is active - manual review required",
                                rule.rule_name,
                            ),
                        });
                    }
                }
                _ => {}
            }
        }

        Ok(alerts)
    }

    async fn log_alert(
        &self, tenant_id: Uuid, transaction_id: Option<Uuid>,
        alert: &ComplianceAlert,
    ) -> Result<(), FbError> {
        sqlx::query(
            "INSERT INTO compliance_alerts (tenant_id, transaction_id, alert_type, \
             alert_severity, alert_message, source) VALUES ($1, $2, $3, $4, $5, 'compliance_engine')"
        )
        .bind(tenant_id)
        .bind(transaction_id)
        .bind(&alert.alert_type)
        .bind(&alert.severity)
        .bind(&alert.message)
        .execute(&self.pool)
        .await
        .map_err(map_db_err)?;
        Ok(())
    }
}

#[async_trait]
impl ComplianceChecker for ComplianceEngine {
    async fn check(
        &self, tenant_id: Uuid, counterparty: &str, instrument_type: &str,
        _desk: &str, amount: Decimal, trade_date: NaiveDate,
    ) -> Result<ComplianceCheckResult, FbError> {
        let sanctions_alerts = self.sanctions_check(
            tenant_id, counterparty, instrument_type, amount, trade_date,
        ).await?;

        let aml_alerts = self.aml_check(
            tenant_id, counterparty, instrument_type, amount, trade_date,
        ).await?;

        let mut all_alerts = Vec::new();
        all_alerts.extend(sanctions_alerts);
        all_alerts.extend(aml_alerts);

        for alert in &all_alerts {
            self.log_alert(tenant_id, None, alert).await?;
        }

        let blocked = all_alerts.iter().any(|a|
            a.severity == "critical" || a.severity == "high"
        );

        Ok(ComplianceCheckResult {
            passed: !blocked,
            blocked,
            alerts: all_alerts,
        })
    }
}
