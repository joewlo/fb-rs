use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use async_trait::async_trait;
use uuid::Uuid;
use crate::kernel::*;

pub struct InMemoryRegistry {
    contracts: RwLock<HashMap<String, Arc<dyn Contract + Send + Sync>>>,
    schemas: RwLock<HashMap<String, InstrumentSchema>>,
}

impl InMemoryRegistry {
    pub fn new() -> Self {
        Self { contracts: RwLock::new(HashMap::new()), schemas: RwLock::new(HashMap::new()) }
    }
}

#[async_trait]
impl ContractRegistry for InMemoryRegistry {
    async fn get_contract(&self, _tenant_id: Uuid, instrument_type: &str) -> Result<Arc<dyn Contract + Send + Sync>, FbError> {
        self.contracts.read().unwrap().get(instrument_type).cloned().ok_or(FbError::ContractNotFound)
    }

    async fn get_schema(&self, _tenant_id: Uuid, instrument_type: &str) -> Result<InstrumentSchema, FbError> {
        self.schemas.read().unwrap().get(instrument_type).cloned().ok_or(FbError::SchemaNotFound)
    }

    fn register_contract(&mut self, instrument_type: &str, contract: Arc<dyn Contract + Send + Sync>) {
        let schema = contract.schema();
        self.contracts.write().unwrap().insert(instrument_type.to_string(), contract);
        self.schemas.write().unwrap().insert(instrument_type.to_string(), schema);
    }

    async fn list_instrument_types(&self, _tenant_id: Uuid) -> Result<Vec<String>, FbError> {
        Ok(self.contracts.read().unwrap().keys().cloned().collect())
    }
}
