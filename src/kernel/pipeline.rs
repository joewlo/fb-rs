use std::sync::Arc;
use rust_decimal::Decimal;
use chrono::Utc;
use uuid::Uuid;
use tracing::info;

use super::types::*;
use super::attribute_bag::AttributeBag;
use super::models::*;
use super::event::*;
use super::errors::FbError;
use super::engine::*;

pub struct PostingEngineImpl {
    pub registry: Box<dyn ContractRegistry + Send + Sync>,
    pub templates: Arc<dyn TemplateEngine + Send + Sync>,
    pub entry_writer: Arc<dyn EntryWriter + Send + Sync>,
    pub fee_engine: Option<Box<dyn FeeCalculator + Send + Sync>>,
    pub position_tracker: Option<Box<dyn PositionTracker + Send + Sync>>,
    pub compliance_checker: Option<Box<dyn ComplianceChecker + Send + Sync>>,
    pub event_store: Option<Box<dyn EventStore + Send + Sync>>,
    pub event_pub: Option<Box<dyn EventPublisher + Send + Sync>>,
}

#[async_trait::async_trait]
impl PostingEngine for PostingEngineImpl {
    async fn submit(&self, mut raw: RawTransaction) -> Result<PostedTransaction, FbError> {
        // Stage 1: Ingest
        raw = self.ingest(raw).await?;

        // Stage 2: Validate + Compliance
        self.validate(&raw).await?;
        self.run_compliance(&raw).await?;

        // Stage 3: Enrich
        let enriched = self.enrich(&raw).await?;

        // Stage 3.5: Fee calculation
        let mut posted = self.generate(&enriched).await?;
        self.apply_fees(&enriched, &mut posted).await?;

        // Stage 4: Check balance
        self.check(&posted).await?;

        // Stage 5: Post
        self.post(&mut posted).await?;

        Ok(posted)
    }

    async fn ingest(&self, mut raw: RawTransaction) -> Result<RawTransaction, FbError> {
        if raw.instrument_type.is_empty() { return Err(FbError::ingest("unknown", "instrument_type required")); }
        if raw.instrument_id.is_empty() { return Err(FbError::ingest("unknown", "instrument_id required")); }
        Ok(raw)
    }

    async fn validate(&self, raw: &RawTransaction) -> Result<(), FbError> {
        let contract = self.registry.get_contract(raw.tenant_id, &raw.instrument_type).await?;
        let errs = contract.validate(raw);
        if !errs.is_empty() {
            return Err(FbError::validate("tx", format!("{}: {}", errs[0].field, errs[0].message)));
        }
        Ok(())
    }

    async fn enrich(&self, raw: &RawTransaction) -> Result<EnrichedTransaction, FbError> {
        let contract = self.registry.get_contract(raw.tenant_id, &raw.instrument_type).await?;
        let derived = contract.enrich(raw).await?;
        Ok(EnrichedTransaction {
            raw: raw.clone(),
            derived_attributes: Some(derived),
            enricher_name: contract.name().to_string(),
            enricher_version: contract.schema().version,
            contract_version: None,
        })
    }

    async fn generate(&self, enriched: &EnrichedTransaction) -> Result<PostedTransaction, FbError> {
        let contract = self.registry.get_contract(enriched.raw.tenant_id, &enriched.raw.instrument_type).await?;
        let rules = contract.posting_rules();

        let (entries, subledger_entries) = self.templates.generate_entries(&rules.template, enriched).await?;

        let tx = Transaction {
            id: Uuid::new_v4(),
            tenant_id: enriched.raw.tenant_id,
            instrument_type: enriched.raw.instrument_type.clone(),
            instrument_id: enriched.raw.instrument_id.clone(),
            parent_tx_id: enriched.raw.parent_tx_id,
            root_tx_id: enriched.raw.root_tx_id.unwrap_or_else(Uuid::new_v4),
            link_type: enriched.raw.link_type.clone(),
            link_depth: enriched.raw.link_depth,
            input_attributes: enriched.raw.attributes.clone(),
            derived_attributes: enriched.derived_attributes.clone(),
            enricher_name: enriched.enricher_name.clone(),
            enricher_version: enriched.enricher_version.clone(),
            contract_version: enriched.contract_version.clone(),
            status: TransactionStatus::Validated,
            idempotency_key: enriched.raw.idempotency_key,
            error_reason: None,
            metadata: None,
            created_at: Utc::now(),
            posted_at: None,
        };

        let mut entries = entries;
        let mut subledger_entries = subledger_entries;
        for e in &mut entries { e.transaction_id = tx.id; e.tenant_id = tx.tenant_id; }
        for s in &mut subledger_entries { s.transaction_id = tx.id; s.tenant_id = tx.tenant_id; }

        Ok(PostedTransaction {
            transaction: tx,
            entries,
            subledger_entries,
            child_txs: vec![],
        })
    }

    async fn check(&self, posted: &PostedTransaction) -> Result<(), FbError> {
        if posted.entries.is_empty() {
            return Err(FbError::check("tx", "empty entries"));
        }
        verify_balance(&posted.entries)
    }

    async fn post(&self, posted: &mut PostedTransaction) -> Result<(), FbError> {
        posted.transaction.status = TransactionStatus::Posted;
        posted.transaction.posted_at = Some(Utc::now());

        let now = Utc::now();
        for e in &mut posted.entries { e.posted_at = now; }

        self.entry_writer.write_entries(&posted.entries, &posted.subledger_entries).await?;

        info!(tx_id = %posted.transaction.id, entries = posted.entries.len(), "transaction posted");
        Ok(())
    }
}

impl PostingEngineImpl {
    async fn run_compliance(&self, raw: &RawTransaction) -> Result<(), FbError> {
        if let Some(ref checker) = self.compliance_checker {
            let cp = raw.attributes.get_string("counterparty").unwrap_or_default();
            let desk = raw.attributes.get_string("desk").unwrap_or_default();
            let amount = raw.attributes.get_quantity(QuantityType::Traded)
                .or_else(|| raw.attributes.get_amount(AmountType::Gross))
                .unwrap_or(Decimal::ZERO);
            let trade_date = raw.attributes.get_date(DateType::Trade).unwrap_or_else(|| chrono::Utc::now().date_naive());
            let result = checker.check(raw.tenant_id, &cp, &raw.instrument_type, &desk, amount, trade_date).await?;
            if result.blocked {
                return Err(FbError::validate("compliance", format!("blocked: {} alerts", result.alerts.len())));
            }
        }
        Ok(())
    }

    async fn apply_fees(&self, enriched: &EnrichedTransaction, posted: &mut PostedTransaction) -> Result<(), FbError> {
        if let Some(ref fee_engine) = self.fee_engine {
            let basis = enriched.derived_attributes.as_ref()
                .and_then(|d| d.get_amount(AmountType::Settlement))
                .unwrap_or(Decimal::ZERO);
            if basis > Decimal::ZERO {
                let fees = fee_engine.calculate(
                    &enriched.raw.tenant_id.to_string(),
                    &enriched.raw.instrument_type,
                    basis,
                ).await?;
                let children = fee_engine.to_child_transactions(&fees, &posted.transaction);
                posted.child_txs.extend(children);
            }
        }
        Ok(())
    }
}

pub fn verify_balance(entries: &[JournalEntry]) -> Result<(), FbError> {
    let mut total_debit = Decimal::ZERO;
    let mut total_credit = Decimal::ZERO;

    for e in entries {
        match e.side {
            Side::Debit => total_debit += e.amount,
            Side::Credit => total_credit += e.amount,
        }
    }

    if total_debit != total_credit {
        return Err(FbError::Unbalanced);
    }
    Ok(())
}
