use rust_decimal::Decimal;
use chrono::{NaiveDate, DateTime, Utc};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use super::types::*;
use super::attribute_bag::AttributeBag;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub instrument_type: String,
    pub instrument_id: String,
    pub parent_tx_id: Option<Uuid>,
    pub root_tx_id: Uuid,
    pub link_type: Option<String>,
    pub link_depth: i32,
    pub input_attributes: AttributeBag,
    pub derived_attributes: Option<AttributeBag>,
    pub enricher_name: String,
    pub enricher_version: String,
    pub contract_version: Option<String>,
    pub status: TransactionStatus,
    pub idempotency_key: Option<Uuid>,
    pub error_reason: Option<String>,
    pub metadata: Option<AttributeBag>,
    pub created_at: DateTime<Utc>,
    pub posted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub transaction_id: Uuid,
    pub entry_sequence: i32,
    pub account_id: Uuid,
    pub amount: Decimal,
    pub currency: String,
    pub side: Side,
    pub value_date: NaiveDate,
    pub narrative: String,
    pub metadata: Option<AttributeBag>,
    pub posted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubledgerEntry {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub transaction_id: Uuid,
    pub subledger_type: SubledgerType,
    pub journal_entry_id: Uuid,
    pub instrument_id: Option<String>,
    pub quantity: Option<Decimal>,
    pub quantity_type: Option<QuantityType>,
    pub price: Option<Decimal>,
    pub counterparty: Option<String>,
    pub trade_date: Option<NaiveDate>,
    pub settle_date: Option<NaiveDate>,
    pub metadata: Option<AttributeBag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub geo: String,
    pub account_code: String,
    pub account_name: String,
    pub display_name: Option<String>,
    pub display_code: Option<String>,
    pub account_type: AccountType,
    pub subledger_type: Option<SubledgerType>,
    pub currency: String,
    pub balance: Decimal,
    pub frozen_balance: Decimal,
    pub version: i64,
    pub sequence_number: i64,
    pub status: String,
    pub attributes: Option<AttributeBag>,
    pub contract_name: Option<String>,
    pub contract_version: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: Uuid,
    pub name: String,
    pub short_code: String,
    pub status: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentSchema {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub instrument_type: String,
    pub schema_data: serde_json::Value,
    pub enricher_name: String,
    pub version: String,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostingRule {
    pub side: Side,
    pub account_code_template: String,
    pub amount_ref: Option<String>,
    pub quantity_ref: Option<String>,
    pub currency: Option<String>,
    pub date_ref: Option<String>,
    pub narrative: Option<String>,
    pub subledger_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostingTemplate {
    pub name: String,
    pub version: String,
    pub entries: Vec<PostingRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostingRules {
    pub template: PostingTemplate,
    pub link_rules: Vec<LinkRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkRule {
    pub link_type: LinkType,
    pub input_mapping: std::collections::HashMap<String, String>,
    pub posting_template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantityPosition {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub account_id: Uuid,
    pub instrument_id: String,
    pub quantity_type: QuantityType,
    pub quantity: Decimal,
    pub version: i64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveContext {
    pub tenant_id: Uuid,
    pub geo: String,
    pub desk: String,
    pub currency: String,
    pub attributes: Option<AttributeBag>,
}
