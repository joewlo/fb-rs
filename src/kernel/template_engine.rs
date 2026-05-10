use std::collections::HashMap;
use std::sync::RwLock;
use async_trait::async_trait;
use uuid::Uuid;
use super::types::*;
use super::attribute_bag::AttributeBag;
use super::models::*;
use super::event::EnrichedTransaction;
use super::engine::*;
use super::errors::FbError;
use rust_decimal::Decimal;

pub struct InMemoryTemplateEngine {
    pub resolver: Box<dyn AccountResolver>,
    templates: RwLock<HashMap<String, PostingTemplate>>,
}

impl InMemoryTemplateEngine {
    pub fn new(resolver: Box<dyn AccountResolver>) -> Self {
        Self { resolver, templates: RwLock::new(HashMap::new()) }
    }
    pub fn register(&self, tmpl: PostingTemplate) {
        self.templates.write().unwrap().insert(tmpl.name.clone(), tmpl);
    }
}

#[async_trait]
impl TemplateEngine for InMemoryTemplateEngine {
    async fn generate_entries(&self, template: &PostingTemplate, enriched: &EnrichedTransaction) -> Result<(Vec<JournalEntry>, Vec<SubledgerEntry>), FbError> {
        let desk = enriched.raw.attributes.get_string("desk").unwrap_or_default();
        let inst_id = &enriched.raw.instrument_id;
        let tenant_id = enriched.raw.tenant_id;
        let geo = enriched.raw.attributes.get_string("geo").unwrap_or_default();

        let resolve_ctx = ResolveContext {
            tenant_id,
            geo: geo.clone(),
            desk: desk.clone(),
            currency: enriched.raw.attributes.get_string("currency").unwrap_or_else(|| "USD".to_string()),
            attributes: None, // skip expensive merge clone
        };

        let mut entries = Vec::with_capacity(template.entries.len());

        // Pre-build substitution map once
        let mut sub_map: HashMap<&str, &str> = HashMap::with_capacity(8);
        sub_map.insert("instrument_id", inst_id.as_str());
        sub_map.insert("desk", desk.as_str());
        for (k, v) in &enriched.raw.attributes.strings { sub_map.insert(k.as_str(), v.as_str()); }
        if let Some(ref d) = enriched.derived_attributes {
            for (k, v) in &d.strings { sub_map.insert(k.as_str(), v.as_str()); }
        }

        for rule in &template.entries {
            let code_template = fast_replace_ref(&rule.account_code_template, &sub_map);
            let account_id = self.resolver.resolve(&code_template, &resolve_ctx).await?;
            let amount = resolve_amount(&rule.amount_ref, enriched);

            entries.push(JournalEntry {
                id: Uuid::new_v4(), tenant_id,
                transaction_id: Uuid::nil(),
                entry_sequence: entries.len() as i32 + 1,
                account_id, amount,
                currency: rule.currency.as_deref().unwrap_or("USD").to_string(),
                side: rule.side,
                value_date: chrono::Utc::now().date_naive(),
                narrative: rule.narrative.clone().unwrap_or_default(),
                metadata: None, posted_at: chrono::Utc::now(),
            });
        }
        Ok((entries, vec![]))
    }

    async fn resolve_account(&self, code_template: &str, ctx: &ResolveContext) -> Result<Uuid, FbError> {
        self.resolver.resolve(code_template, ctx).await
    }

    async fn load_template(&self, name: &str) -> Result<PostingTemplate, FbError> {
        self.templates.read().unwrap().get(name).cloned().ok_or(FbError::ContractNotFound)
    }
}

/// Fast single-pass template substitution with &str references — zero allocation.
fn fast_replace_ref(template: &str, map: &HashMap<&str, &str>) -> String {
    let mut result = String::with_capacity(template.len());
    let bytes = template.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'{' {
            if let Some(end) = bytes[i..].iter().position(|&b| b == b'}') {
                let key = &template[i + 1..i + end];
                if let Some(val) = map.get(key) {
                    result.push_str(val);
                    i += end + 1;
                    continue;
                }
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

fn resolve_amount(ref_opt: &Option<String>, enriched: &EnrichedTransaction) -> Decimal {
    let ref_str = match ref_opt { Some(s) => s.as_str(), None => return Decimal::ZERO };
    if let Some(ref d) = enriched.derived_attributes {
        if let Some(name) = ref_str.strip_prefix("$derived.amount.") {
            let at = amount_type_from_str(name);
            if let Some(v) = d.get_amount(at) { return v; }
        }
        if let Some(name) = ref_str.strip_prefix("$input.quantity.") {
            let qt = quantity_type_from_str(name);
            if let Some(v) = enriched.raw.attributes.get_quantity(qt) { return v; }
        }
    }
    Decimal::ZERO
}

fn amount_type_from_str(s: &str) -> AmountType {
    match s {
        "gross" => AmountType::Gross, "net" => AmountType::Net,
        "settlement" => AmountType::Settlement, "commission" => AmountType::Commission,
        "tax" => AmountType::Tax, "fee" => AmountType::Fee,
        "accrued_interest" => AmountType::AccruedInterest, "stamp_duty" => AmountType::StampDuty,
        _ => AmountType::Gross,
    }
}

fn quantity_type_from_str(s: &str) -> QuantityType {
    match s {
        "current" => QuantityType::Current, "traded" => QuantityType::Traded,
        "safe_keeping" => QuantityType::SafeKeeping, "segregated" => QuantityType::Segregated,
        "frozen" => QuantityType::Frozen, "available" => QuantityType::Available,
        "pending_settlement" => QuantityType::PendingSettlement, "pledged" => QuantityType::Pledged,
        "recalled" => QuantityType::Recalled, "accrued_interest" => QuantityType::AccruedInterest,
        _ => QuantityType::Traded,
    }
}
