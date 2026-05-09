use std::sync::Mutex;
use async_trait::async_trait;
use rust_decimal::Decimal;
use super::models::*;
use super::engine::*;
use super::errors::FbError;

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
