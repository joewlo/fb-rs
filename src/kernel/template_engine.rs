use std::collections::HashMap;
use std::sync::Mutex;
use async_trait::async_trait;
use uuid::Uuid;
use super::models::*;
use super::event::EnrichedTransaction;
use super::engine::*;
use super::errors::FbError;

pub struct InMemoryTemplateEngine {
    pub resolver: Box<dyn AccountResolver>,
    templates: Mutex<HashMap<String, PostingTemplate>>,
}

impl InMemoryTemplateEngine {
    pub fn new(resolver: Box<dyn AccountResolver>) -> Self {
        Self { resolver, templates: Mutex::new(HashMap::new()) }
    }
    pub fn register(&self, tmpl: PostingTemplate) {
        self.templates.lock().unwrap().insert(tmpl.name.clone(), tmpl);
    }
}

#[async_trait]
impl TemplateEngine for InMemoryTemplateEngine {
    async fn generate_entries(&self, template: &PostingTemplate, enriched: &EnrichedTransaction) -> Result<(Vec<JournalEntry>, Vec<SubledgerEntry>), FbError> {
        let mut entries = Vec::new();
        let mut sub_entries = Vec::new();
        let resolve_ctx = ResolveContext {
            tenant_id: enriched.raw.tenant_id,
            geo: enriched.raw.attributes.get_string("geo").unwrap_or_default(),
            desk: enriched.raw.attributes.get_string("desk").unwrap_or_default(),
            currency: enriched.raw.attributes.get_string("currency").unwrap_or_else(|| "USD".to_string()),
            attributes: Some(enriched.raw.attributes.merge(enriched.derived_attributes.as_ref().unwrap_or(&Default::default()))),
        };

        for rule in &template.entries {
            let code_template = Self::substitute(&rule.account_code_template, enriched);
            let account_id = self.resolver.resolve(&code_template, &resolve_ctx).await?;
            let amount = Self::resolve_amount(&rule.amount_ref, enriched);

            let entry = JournalEntry {
                id: Uuid::new_v4(),
                tenant_id: enriched.raw.tenant_id,
                transaction_id: Uuid::nil(),
                entry_sequence: (entries.len() + 1) as i32,
                account_id,
                amount,
                currency: rule.currency.as_deref().unwrap_or("USD").to_string(),
                side: rule.side,
                value_date: chrono::Utc::now().date_naive(),
                narrative: rule.narrative.clone().unwrap_or_default(),
                metadata: None,
                posted_at: chrono::Utc::now(),
            };
            entries.push(entry);
        }
        Ok((entries, sub_entries))
    }

    async fn resolve_account(&self, code_template: &str, ctx: &ResolveContext) -> Result<Uuid, FbError> {
        self.resolver.resolve(code_template, ctx).await
    }

    async fn load_template(&self, name: &str) -> Result<PostingTemplate, FbError> {
        self.templates.lock().unwrap().get(name).cloned().ok_or(FbError::ContractNotFound)
    }
}

impl InMemoryTemplateEngine {
    fn substitute(template: &str, enriched: &EnrichedTransaction) -> String {
        let mut result = template.to_string();
        result = result.replace("{instrument_id}", &enriched.raw.instrument_id);
        result = result.replace("{instrument_type}", &enriched.raw.instrument_type);
        for (k, v) in &enriched.raw.attributes.strings { result = result.replace(&format!("{{{}}}", k), v); }
        if let Some(ref d) = enriched.derived_attributes {
            for (k, v) in &d.strings { result = result.replace(&format!("{{{}}}", k), v); }
        }
        result
    }

    fn resolve_amount(ref_opt: &Option<String>, enriched: &EnrichedTransaction) -> rust_decimal::Decimal {
        let ref_str = match ref_opt { Some(s) => s, None => return rust_decimal::Decimal::ZERO };
        if let Some(ref d) = enriched.derived_attributes {
            if let Some(amount_name) = ref_str.strip_prefix("$derived.amount.") {
                if let Ok(at) = serde_json::from_str(&format!("\"{}\"", amount_name)) {
                    if let Some(v) = d.get_amount(at) { return v; }
                }
            }
            if let Some(qty_name) = ref_str.strip_prefix("$input.quantity.") {
                if let Ok(qt) = serde_json::from_str(&format!("\"{}\"", qty_name)) {
                    if let Some(v) = enriched.raw.attributes.get_quantity(qt) { return v; }
                }
            }
        }
        rust_decimal::Decimal::ZERO
    }
}
