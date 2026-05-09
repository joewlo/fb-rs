use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct JournalEntry {
    pub id: Uuid,
    pub tenant_id: String,
    pub entry_date: DateTime<Utc>,
    pub account: String,
    pub amount: Decimal,
    pub currency: String,
    pub description: String,
}

#[derive(Debug, thiserror::Error)]
pub enum BatchWriterError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("serialization error: {0}")]
    Serialize(String),
}

pub struct BatchEntryWriter {
    pool: PgPool,
    buffer: Vec<JournalEntry>,
    batch_size: usize,
}

impl BatchEntryWriter {
    pub fn new(pool: PgPool, batch_size: usize) -> Self {
        Self {
            pool,
            buffer: Vec::with_capacity(batch_size),
            batch_size,
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn buffer_size(&self) -> usize {
        self.batch_size
    }

    pub fn push(&mut self, entry: JournalEntry) {
        self.buffer.push(entry);
    }

    pub fn needs_flush(&self) -> bool {
        self.buffer.len() >= self.batch_size
    }

    pub async fn flush(&mut self) -> Result<usize, BatchWriterError> {
        if self.buffer.is_empty() {
            return Ok(0);
        }

        let count = self.buffer.len();
        let mut tx = self.pool.begin().await?;

        for entry in &self.buffer {
            let amount_str = entry.amount.to_string();

            sqlx::query(
                "INSERT INTO journal_entries (id, tenant_id, entry_date, account, amount, currency, description) \
                 VALUES ($1, $2, $3, $4, $5::numeric, $6, $7)",
            )
            .bind(entry.id)
            .bind(&entry.tenant_id)
            .bind(entry.entry_date)
            .bind(&entry.account)
            .bind(&amount_str)
            .bind(&entry.currency)
            .bind(&entry.description)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        self.buffer.clear();
        Ok(count)
    }
}

impl Drop for BatchEntryWriter {
    fn drop(&mut self) {
        if !self.buffer.is_empty() {
            tracing::warn!(
                count = self.buffer.len(),
                "BatchEntryWriter dropped with unflushed entries"
            );
        }
    }
}
