use rust_decimal::Decimal;
use chrono::{NaiveDate, DateTime, Utc};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use super::attribute_bag::AttributeBag;
use super::models::*;
use super::types::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: i64,
    pub tenant_id: Uuid,
    pub event_id: Uuid,
    pub aggregate_type: String,
    pub aggregate_id: Uuid,
    pub event_type: String,
    pub event_data: serde_json::Value,
    pub metadata: serde_json::Value,
    pub version: i64,
    pub created_at: DateTime<Utc>,
}

pub const EVENT_TRANSACTION_POSTED: &str = "TransactionPosted";
pub const EVENT_TRANSACTION_FAILED: &str = "TransactionFailed";
pub const EVENT_TRANSACTION_CANCELLED: &str = "TransactionCancelled";
pub const EVENT_TRANSACTION_ENRICHED: &str = "TransactionEnriched";
pub const EVENT_ACCOUNT_OPENED: &str = "AccountOpened";
pub const EVENT_ACCOUNT_FROZEN: &str = "AccountFrozen";
pub const EVENT_ACCOUNT_CLOSED: &str = "AccountClosed";
pub const EVENT_POSITION_UPDATED: &str = "PositionUpdated";
pub const EVENT_BALANCE_UPDATED: &str = "BalanceUpdated";
pub const EVENT_CHILD_TRANSACTION_POSTED: &str = "ChildTransactionPosted";
pub const AGGREGATE_TRANSACTION: &str = "Transaction";
pub const AGGREGATE_ACCOUNT: &str = "Account";

pub const STAGE_INGEST: &str = "INGEST";
pub const STAGE_VALIDATE: &str = "VALIDATE";
pub const STAGE_ENRICH: &str = "ENRICH";
pub const STAGE_GENERATE: &str = "GENERATE";
pub const STAGE_CHECK: &str = "CHECK";

#[derive(Debug, Clone)]
pub struct RawTransaction {
    pub tenant_id: Uuid,
    pub instrument_type: String,
    pub instrument_id: String,
    pub parent_tx_id: Option<Uuid>,
    pub root_tx_id: Option<Uuid>,
    pub link_type: Option<String>,
    pub link_depth: i32,
    pub attributes: AttributeBag,
    pub idempotency_key: Option<Uuid>,
    pub metadata: Option<AttributeBag>,
}

#[derive(Debug, Clone)]
pub struct EnrichedTransaction {
    pub raw: RawTransaction,
    pub derived_attributes: Option<AttributeBag>,
    pub enricher_name: String,
    pub enricher_version: String,
    pub contract_version: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PostedTransaction {
    pub transaction: Transaction,
    pub entries: Vec<JournalEntry>,
    pub subledger_entries: Vec<SubledgerEntry>,
    pub child_txs: Vec<Transaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceSummary {
    pub total_debit: Decimal,
    pub total_credit: Decimal,
    pub balanced: bool,
}
