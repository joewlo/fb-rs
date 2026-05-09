use std::sync::Mutex;
use async_trait::async_trait;
use sqlx::PgPool;
use crate::kernel::*;

pub struct SimpleEntryWriter {
    pub entries: Mutex<Vec<JournalEntry>>,
}

impl SimpleEntryWriter {
    pub fn new() -> Self { Self { entries: Mutex::new(Vec::new()) } }
}

#[async_trait]
impl EntryWriter for SimpleEntryWriter {
    async fn write_entries(&self, entries: &[JournalEntry], _sub: &[SubledgerEntry]) -> Result<(), FbError> {
        self.entries.lock().unwrap().extend_from_slice(entries);
        Ok(())
    }
}

pub struct PgEntryWriter {
    pool: PgPool,
    batch_size: usize,
    buffer: Mutex<Vec<JournalEntry>>,
}

impl PgEntryWriter {
    pub fn new(pool: PgPool) -> Self { Self { pool, batch_size: 500, buffer: Mutex::new(Vec::with_capacity(500)) } }

    pub async fn flush(&self) -> Result<(), FbError> {
        let entries: Vec<JournalEntry> = std::mem::take(&mut *self.buffer.lock().unwrap());
        if entries.is_empty() { return Ok(()); }

        for e in &entries {
            let amt = e.amount.to_string();
            let side = match e.side { Side::Debit => "DEBIT", Side::Credit => "CREDIT" };
            sqlx::query("INSERT INTO journal_entries (id,tenant_id,transaction_id,entry_sequence,account_id,amount,currency,side,value_date,narrative,metadata,posted_at) VALUES ($1,$2,$3,$4,$5,$6::numeric,$7,$8,$9,$10,'{}',$11)")
                .bind(e.id).bind(e.tenant_id).bind(e.transaction_id).bind(e.entry_sequence)
                .bind(e.account_id).bind(&amt).bind(&e.currency).bind(side)
                .bind(e.value_date).bind(&e.narrative).bind(e.posted_at)
                .execute(&self.pool).await.map_err(|err| FbError::pipeline("POST", &e.id.to_string(), err))?;
        }
        Ok(())
    }
}

#[async_trait]
impl EntryWriter for PgEntryWriter {
    async fn write_entries(&self, entries: &[JournalEntry], _sub: &[SubledgerEntry]) -> Result<(), FbError> {
        let should_flush = {
            let mut buf = self.buffer.lock().unwrap();
            buf.extend_from_slice(entries);
            buf.len() >= self.batch_size
        };
        if should_flush { self.flush().await?; }
        Ok(())
    }
}
