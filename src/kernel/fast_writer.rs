use std::sync::Mutex;
use async_trait::async_trait;
use sqlx::PgPool;
use crate::kernel::*;

pub struct FastBatchWriter {
    pool: PgPool,
    buffer: Mutex<Vec<JournalEntry>>,
}

impl FastBatchWriter {
    pub fn new(pool: PgPool) -> Self { Self { pool, buffer: Mutex::new(Vec::with_capacity(1024)) } }

    pub async fn flush(&self) -> Result<(), FbError> {
        let entries: Vec<JournalEntry> = std::mem::take(&mut *self.buffer.lock().unwrap());
        if entries.is_empty() { return Ok(()); }

        for chunk in entries.chunks(500) {
            let mut sql = String::with_capacity(4096);
            sql.push_str("INSERT INTO journal_entries (id,tenant_id,transaction_id,entry_sequence,account_id,amount,currency,side,value_date,narrative,metadata,posted_at) VALUES ");
            
            // Pre-build all string representations outside the query builder
            let params: Vec<(String, String)> = chunk.iter().map(|e| {
                (e.amount.to_string(), match e.side { Side::Debit => "DEBIT".to_string(), Side::Credit => "CREDIT".to_string() })
            }).collect();

            for (i, e) in chunk.iter().enumerate() {
                if i > 0 { sql.push(','); }
                let b = i * 11 + 1;
                sql.push_str(&format!("(${b}::uuid,${}::uuid,${}::uuid,${},${}::uuid,${}::numeric,${},${},${},${},'{{}}',${})",
                    b+1, b+2, b+3, b+4, b+5, b+6, b+7, b+8, b+9, b+10));
            }

            let mut q = sqlx::query(&sql);
            for (e, (amt, side)) in chunk.iter().zip(params.iter()) {
                q = q.bind(e.id).bind(e.tenant_id).bind(e.transaction_id).bind(e.entry_sequence)
                    .bind(e.account_id).bind(amt).bind(&e.currency).bind(side)
                    .bind(e.value_date).bind(&e.narrative).bind(e.posted_at);
            }
            q.execute(&self.pool).await.map_err(|e| FbError::pipeline("POST", "batch", e))?;
        }
        Ok(())
    }
}

#[async_trait]
impl EntryWriter for FastBatchWriter {
    async fn write_entries(&self, entries: &[JournalEntry], _sub: &[SubledgerEntry]) -> Result<(), FbError> {
        let should_flush = {
            let mut buf = self.buffer.lock().unwrap();
            buf.extend_from_slice(entries);
            buf.len() >= 1000
        };
        if should_flush { self.flush().await?; }
        Ok(())
    }
}
