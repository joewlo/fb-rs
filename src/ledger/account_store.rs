use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::kernel::*;

#[derive(Debug, Clone, FromRow)]
struct AccountRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub geo: String,
    pub account_code: String,
    pub account_name: String,
    pub display_name: Option<String>,
    pub display_code: Option<String>,
    pub account_type: String,
    pub subledger_type: Option<String>,
    pub currency: String,
    pub balance: Decimal,
    pub frozen_balance: Decimal,
    pub version: i64,
    pub sequence_number: i64,
    pub status: String,
    pub attributes: Option<serde_json::Value>,
    pub contract_name: Option<String>,
    pub contract_version: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn parse_account_type(s: &str) -> AccountType {
    match s {
        "ASSET" => AccountType::Asset,
        "LIABILITY" => AccountType::Liability,
        "EQUITY" => AccountType::Equity,
        "INCOME" => AccountType::Income,
        "EXPENSE" => AccountType::Expense,
        _ => AccountType::Asset,
    }
}

fn parse_subledger_type(s: &str) -> SubledgerType {
    match s {
        "TRADING" => SubledgerType::Trading,
        "CASH" => SubledgerType::Cash,
        "PNL" => SubledgerType::PNL,
        "SETTLEMENT" => SubledgerType::Settlement,
        "POSITION" => SubledgerType::Position,
        _ => SubledgerType::Trading,
    }
}

impl From<AccountRow> for Account {
    fn from(r: AccountRow) -> Self {
        Account {
            id: r.id,
            tenant_id: r.tenant_id,
            geo: r.geo,
            account_code: r.account_code,
            account_name: r.account_name,
            display_name: r.display_name,
            display_code: r.display_code,
            account_type: parse_account_type(&r.account_type),
            subledger_type: r.subledger_type.as_deref().map(parse_subledger_type),
            currency: r.currency,
            balance: r.balance,
            frozen_balance: r.frozen_balance,
            version: r.version,
            sequence_number: r.sequence_number,
            status: r.status,
            attributes: r.attributes.and_then(|v| serde_json::from_value(v).ok()),
            contract_name: r.contract_name,
            contract_version: r.contract_version,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

fn map_db_err(e: sqlx::Error) -> FbError {
    FbError::pipeline("DB", "account", e)
}

pub struct AccountStore {
    pool: PgPool,
}

impl AccountStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn resolve_template(template: &str, ctx: &ResolveContext) -> String {
        let mut result = template.to_string();
        result = result.replace("{tenant_id}", &ctx.tenant_id.to_string());
        result = result.replace("{geo}", &ctx.geo);
        result = result.replace("{desk}", &ctx.desk);
        result = result.replace("{currency}", &ctx.currency);
        if let Some(ref attrs) = ctx.attributes {
            if let Some(instrument_id) = attrs.get_string("instrument_id") {
                result = result.replace("{instrument_id}", &instrument_id);
            }
            if let Some(instrument_type) = attrs.get_string("instrument_type") {
                result = result.replace("{instrument_type}", &instrument_type);
            }
            if let Some(counterparty) = attrs.get_string("counterparty") {
                result = result.replace("{counterparty}", &counterparty);
            }
        }
        result = result.replace("{instrument_id}", "");
        result = result.replace("{instrument_type}", "");
        result = result.replace("{counterparty}", "");
        result
    }

    pub async fn get_account(&self, id: Uuid) -> Result<Account, FbError> {
        let row: AccountRow = sqlx::query_as(
            "SELECT id, tenant_id, geo, account_code, account_name, display_name, display_code, \
             account_type, subledger_type, currency, balance, frozen_balance, version, \
             sequence_number, status, attributes, contract_name, contract_version, \
             created_at, updated_at FROM accounts WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err)?
        .ok_or(FbError::AccountNotFound)?;
        Ok(row.into())
    }

    pub async fn get_account_by_code(
        &self, tenant_id: Uuid, geo: &str, code: &str,
    ) -> Result<Account, FbError> {
        let row: AccountRow = sqlx::query_as(
            "SELECT id, tenant_id, geo, account_code, account_name, display_name, display_code, \
             account_type, subledger_type, currency, balance, frozen_balance, version, \
             sequence_number, status, attributes, contract_name, contract_version, \
             created_at, updated_at FROM accounts WHERE tenant_id = $1 AND geo = $2 AND account_code = $3"
        )
        .bind(tenant_id)
        .bind(geo)
        .bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err)?
        .ok_or(FbError::AccountNotFound)?;
        Ok(row.into())
    }

    pub async fn create_account(
        &self,
        tenant_id: Uuid,
        geo: &str,
        account_code: &str,
        account_name: &str,
        account_type: AccountType,
        subledger_type: Option<SubledgerType>,
        currency: &str,
        display_name: Option<&str>,
        display_code: Option<&str>,
        contract_name: Option<&str>,
        contract_version: Option<&str>,
    ) -> Result<Account, FbError> {
        let type_str = format!("{:?}", account_type).to_uppercase();
        let subledger_str = subledger_type.map(|s| format!("{:?}", s).to_uppercase());
        let now = Utc::now();
        let row: AccountRow = sqlx::query_as(
            "INSERT INTO accounts (tenant_id, geo, account_code, account_name, display_name, \
             display_code, account_type, subledger_type, currency, balance, frozen_balance, \
             version, sequence_number, status, attributes, contract_name, contract_version, \
             created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 0, 0, 0, 0, 'active', '{}', $10, $11, $12, $12) \
             RETURNING id, tenant_id, geo, account_code, account_name, display_name, display_code, \
             account_type, subledger_type, currency, balance, frozen_balance, version, \
             sequence_number, status, attributes, contract_name, contract_version, \
             created_at, updated_at"
        )
        .bind(tenant_id)
        .bind(geo)
        .bind(account_code)
        .bind(account_name)
        .bind(display_name)
        .bind(display_code)
        .bind(type_str)
        .bind(subledger_str)
        .bind(currency)
        .bind(contract_name)
        .bind(contract_version)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_err)?;
        Ok(row.into())
    }

    pub async fn list_accounts(&self, tenant_id: Uuid) -> Result<Vec<Account>, FbError> {
        let rows: Vec<AccountRow> = sqlx::query_as(
            "SELECT id, tenant_id, geo, account_code, account_name, display_name, display_code, \
             account_type, subledger_type, currency, balance, frozen_balance, version, \
             sequence_number, status, attributes, contract_name, contract_version, \
             created_at, updated_at FROM accounts WHERE tenant_id = $1 ORDER BY account_code"
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }
}

#[async_trait]
impl AccountResolver for AccountStore {
    async fn resolve(&self, code_template: &str, ctx: &ResolveContext) -> Result<Uuid, FbError> {
        let code = Self::resolve_template(code_template, ctx);
        let row: AccountRow = sqlx::query_as(
            "SELECT id, tenant_id, geo, account_code, account_name, display_name, display_code, \
             account_type, subledger_type, currency, balance, frozen_balance, version, \
             sequence_number, status, attributes, contract_name, contract_version, \
             created_at, updated_at FROM accounts WHERE tenant_id = $1 AND geo = $2 AND account_code = $3"
        )
        .bind(ctx.tenant_id)
        .bind(&ctx.geo)
        .bind(&code)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_err)?
        .ok_or(FbError::AccountNotFound)?;
        Ok(row.id)
    }
}
