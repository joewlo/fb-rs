use thiserror::Error;

#[derive(Error, Debug)]
pub enum FbError {
    #[error("invalid number format")]
    InvalidNumber,
    #[error("attribute bag is nil")]
    NilAttributeBag,
    #[error("debits do not equal credits")]
    Unbalanced,
    #[error("duplicate event version")]
    DuplicateEvent,
    #[error("event not found")]
    EventNotFound,
    #[error("version conflict: concurrent modification")]
    VersionConflict,
    #[error("account is frozen")]
    AccountFrozen,
    #[error("account not found")]
    AccountNotFound,
    #[error("tenant not found")]
    TenantNotFound,
    #[error("tenant is not active")]
    TenantInactive,
    #[error("contract not found")]
    ContractNotFound,
    #[error("instrument schema not found")]
    SchemaNotFound,
    #[error("invalid instrument type")]
    InstrumentInvalid,
    #[error("idempotency key already processed")]
    IdempotencyKeyExists,
    #[error("transaction DAG contains a cycle")]
    DagCycle,
    #[error("parent transaction is not posted")]
    ParentNotPosted,
    #[error("cannot void transaction with posted children")]
    ChildExists,
    #[error("unauthorized")]
    Unauthorized,
    #[error("invalid transaction status transition")]
    InvalidStatus,
    #[error("side must be DEBIT or CREDIT")]
    InvalidSide,
    #[error("transaction must have at least one entry")]
    EmptyEntries,
    #[error("[{stage}] {tx_id}: {reason}")]
    PipelineError { stage: String, tx_id: String, reason: String },
}

#[derive(Error, Debug)]
#[error("{field}: {message} ({code})")]
pub struct ValidationError {
    pub field: String,
    pub code: String,
    pub message: String,
}

impl FbError {
    pub fn pipeline(stage: &str, tx_id: &str, reason: impl std::fmt::Display) -> Self {
        FbError::PipelineError { stage: stage.to_string(), tx_id: tx_id.to_string(), reason: reason.to_string() }
    }
    pub fn ingest(tx_id: &str, reason: impl std::fmt::Display) -> Self { Self::pipeline(STAGE_INGEST, tx_id, reason) }
    pub fn validate(tx_id: &str, reason: impl std::fmt::Display) -> Self { Self::pipeline(STAGE_VALIDATE, tx_id, reason) }
    pub fn enrich(tx_id: &str, reason: impl std::fmt::Display) -> Self { Self::pipeline(STAGE_ENRICH, tx_id, reason) }
    pub fn generate(tx_id: &str, reason: impl std::fmt::Display) -> Self { Self::pipeline(STAGE_GENERATE, tx_id, reason) }
    pub fn check(tx_id: &str, reason: impl std::fmt::Display) -> Self { Self::pipeline(STAGE_CHECK, tx_id, reason) }
}

use super::event::*;
