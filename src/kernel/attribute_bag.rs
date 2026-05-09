use std::collections::HashMap;
use rust_decimal::Decimal;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use super::types::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AttributeBag {
    pub quantities: HashMap<QuantityType, Decimal>,
    pub prices: HashMap<PriceType, Decimal>,
    pub amounts: HashMap<AmountType, Decimal>,
    pub dates: HashMap<DateType, NaiveDate>,
    pub strings: HashMap<String, String>,
    pub enums: HashMap<String, String>,
}

impl AttributeBag {
    pub fn new() -> Self {
        Self {
            quantities: HashMap::with_capacity(4),
            prices: HashMap::with_capacity(4),
            amounts: HashMap::with_capacity(8),
            dates: HashMap::with_capacity(4),
            strings: HashMap::with_capacity(8),
            enums: HashMap::with_capacity(4),
        }
    }

    pub fn merge(&self, derived: &AttributeBag) -> AttributeBag {
        let mut merged = self.clone();
        for (k, v) in &derived.quantities { merged.quantities.insert(*k, *v); }
        for (k, v) in &derived.prices { merged.prices.insert(*k, *v); }
        for (k, v) in &derived.amounts { merged.amounts.insert(*k, *v); }
        for (k, v) in &derived.dates { merged.dates.insert(*k, *v); }
        for (k, v) in &derived.strings { merged.strings.insert(k.clone(), v.clone()); }
        for (k, v) in &derived.enums { merged.enums.insert(k.clone(), v.clone()); }
        merged
    }

    pub fn get_quantity(&self, t: QuantityType) -> Option<Decimal> { self.quantities.get(&t).copied() }
    pub fn get_price(&self, t: PriceType) -> Option<Decimal> { self.prices.get(&t).copied() }
    pub fn get_amount(&self, t: AmountType) -> Option<Decimal> { self.amounts.get(&t).copied() }
    pub fn get_date(&self, t: DateType) -> Option<NaiveDate> { self.dates.get(&t).copied() }
    pub fn get_string(&self, key: &str) -> Option<String> { self.strings.get(key).cloned() }
    pub fn get_enum(&self, key: &str) -> Option<String> { self.enums.get(key).cloned() }

    pub fn set_quantity(&mut self, t: QuantityType, v: Decimal) { self.quantities.insert(t, v); }
    pub fn set_price(&mut self, t: PriceType, v: Decimal) { self.prices.insert(t, v); }
    pub fn set_amount(&mut self, t: AmountType, v: Decimal) { self.amounts.insert(t, v); }
    pub fn set_date(&mut self, t: DateType, v: NaiveDate) { self.dates.insert(t, v); }
    pub fn set_string(&mut self, key: &str, value: &str) { self.strings.insert(key.to_string(), value.to_string()); }
    pub fn set_enum(&mut self, key: &str, value: &str) { self.enums.insert(key.to_string(), value.to_string()); }
}
