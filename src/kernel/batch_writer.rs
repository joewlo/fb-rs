use std::sync::Mutex;
use async_trait::async_trait;
use sqlx::PgPool;
use crate::kernel::*;
use crate::kernel::errors::FbError;

pub struct SimpleEntryWriter {
    entries: Mutex<Vec<JournalEntry>>,
}

impl SimpleEntryWriter {
    pub fn new() -> Self {
        Self { entries: Mutex::new(Vec::new()) }
    }
    pub fn take_entries(&self) -> Vec<JournalEntry> {
        std::mem::take(&mut *self.entries.lock().unwrap())
    }
}

#[async_trait]
impl EntryWriter for SimpleEntryWriter {
    async fn write_entries(&self, entries: &[JournalEntry], _subledger: &[SubledgerEntry]) -> Result<(), FbError> {
        self.entries.lock().unwrap().extend_from_slice(entries);
        Ok(())
    }
}

// PgEntryWriter inserts journal entries directly into PostgreSQL.
pub struct PgEntryWriter {
    pool: PgPool,
}

impl PgEntryWriter {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait]
impl EntryWriter for PgEntryWriter {
    async fn write_entries(&self, entries: &[JournalEntry], _subledger: &[SubledgerEntry]) -> Result<(), FbError> {
        for e in entries {
            let amount_str = e.amount.to_string();
            let side_str = match e.side { Side::Debit => "DEBIT", Side::Credit => "CREDIT" };
            sqlx::query(
                "INSERT INTO journal_entries (id, tenant_id, transaction_id, entry_sequence, account_id, amount, currency, side, value_date, narrative, metadata, posted_at) VALUES ($1,$2,$3,$4,$5,$6::numeric,$7,$8,$9,$10,'{}',$11)"
            )
            .bind(e.id).bind(e.tenant_id).bind(e.transaction_id).bind(e.entry_sequence)
            .bind(e.account_id).bind(&amount_str).bind(&e.currency).bind(side_str)
            .bind(e.value_date).bind(&e.narrative).bind(e.posted_at)
            .execute(&self.pool).await
            .map_err(|err| FbError::pipeline("POST", &e.id.to_string(), err))?;
        }
        Ok(())
    }
}
