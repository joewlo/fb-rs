use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use rust_decimal::Decimal;
use chrono::{NaiveDate, DateTime, Utc};
use uuid::Uuid;
use super::types::*;
use super::attribute_bag::AttributeBag;
use super::models::*;
use super::event::*;
use super::errors::{FbError, ValidationError};

// -- PostingEngine trait --
#[async_trait]
pub trait PostingEngine: Send + Sync {
    async fn submit(&self, raw: RawTransaction) -> Result<PostedTransaction, FbError>;
    async fn ingest(&self, raw: RawTransaction) -> Result<RawTransaction, FbError>;
    async fn validate(&self, raw: &RawTransaction) -> Result<(), FbError>;
    async fn enrich(&self, raw: &RawTransaction) -> Result<EnrichedTransaction, FbError>;
    async fn generate(&self, enriched: &EnrichedTransaction) -> Result<PostedTransaction, FbError>;
    async fn check(&self, posted: &PostedTransaction) -> Result<(), FbError>;
    async fn post(&self, posted: &mut PostedTransaction) -> Result<(), FbError>;
}

// -- Contract trait --
#[async_trait]
pub trait Contract: Send + Sync {
    fn name(&self) -> &str;
    fn schema(&self) -> InstrumentSchema;
    fn validate(&self, tx: &RawTransaction) -> Vec<ValidationError>;
    async fn enrich(&self, tx: &RawTransaction) -> Result<AttributeBag, FbError>;
    fn posting_rules(&self) -> PostingRules;
    async fn on_post(&self, tx: &PostedTransaction) -> Result<(), FbError>;
}

// -- Registry --
#[async_trait]
pub trait ContractRegistry: Send + Sync {
    async fn get_contract(&self, tenant_id: Uuid, instrument_type: &str) -> Result<Arc<dyn Contract + Send + Sync>, FbError>;
    async fn get_schema(&self, tenant_id: Uuid, instrument_type: &str) -> Result<InstrumentSchema, FbError>;
    fn register_contract(&mut self, instrument_type: &str, contract: Arc<dyn Contract + Send + Sync>);
    async fn list_instrument_types(&self, tenant_id: Uuid) -> Result<Vec<String>, FbError>;
}

// -- Template Engine --
#[async_trait]
pub trait TemplateEngine: Send + Sync {
    async fn generate_entries(&self, template: &PostingTemplate, enriched: &EnrichedTransaction) -> Result<(Vec<JournalEntry>, Vec<SubledgerEntry>), FbError>;
    async fn resolve_account(&self, code_template: &str, ctx: &ResolveContext) -> Result<Uuid, FbError>;
    async fn load_template(&self, name: &str) -> Result<PostingTemplate, FbError>;
}

// -- Account Service --
#[async_trait]
pub trait AccountResolver: Send + Sync {
    async fn resolve(&self, code_template: &str, ctx: &ResolveContext) -> Result<Uuid, FbError>;
}

// -- Entry Writer --
#[async_trait]
pub trait EntryWriter: Send + Sync {
    async fn write_entries(&self, entries: &[JournalEntry], subledger_entries: &[SubledgerEntry]) -> Result<(), FbError>;
}

// -- Fee Calculator --
#[async_trait]
pub trait FeeCalculator: Send + Sync {
    async fn calculate(&self, tenant_id: &str, instrument_type: &str, basis_amount: Decimal) -> Result<Vec<FeeItem>, FbError>;
    fn to_child_transactions(&self, fees: &[FeeItem], parent: &Transaction) -> Vec<Transaction>;
}

#[derive(Debug, Clone)]
pub struct FeeItem {
    pub fee_code: String,
    pub fee_category: String,
    pub fee_name: String,
    pub fee_amount: Decimal,
    pub currency: String,
}

// -- Position Tracker --
#[async_trait]
pub trait PositionTracker: Send + Sync {
    async fn track_position(&self, tenant_id: Uuid, account_id: Uuid, instrument_id: &str, side: &str, quantity: Decimal, amount: Decimal) -> Result<(), FbError>;
}

// -- Compliance Checker --
#[async_trait]
pub trait ComplianceChecker: Send + Sync {
    async fn check(&self, tenant_id: Uuid, counterparty: &str, instrument_type: &str, desk: &str, amount: Decimal, trade_date: NaiveDate) -> Result<ComplianceCheckResult, FbError>;
}

#[derive(Debug, Clone)]
pub struct ComplianceCheckResult {
    pub passed: bool,
    pub blocked: bool,
    pub alerts: Vec<ComplianceAlert>,
}

#[derive(Debug, Clone)]
pub struct ComplianceAlert {
    pub alert_type: String,
    pub severity: String,
    pub message: String,
}

// -- Event Store --
#[async_trait]
pub trait EventStore: Send + Sync {
    async fn append(&self, tenant_id: Uuid, events: &[Event]) -> Result<(), FbError>;
}

// -- Event Publisher --
#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, event: &Event) -> Result<(), FbError>;
}
