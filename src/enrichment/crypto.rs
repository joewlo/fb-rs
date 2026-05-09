use async_trait::async_trait;
use rust_decimal_macros::dec;
use uuid::Uuid;
use crate::kernel::*;

pub struct CryptoContract;

#[async_trait]
impl Contract for CryptoContract {
    fn name(&self) -> &str { "crypto" }
    fn schema(&self) -> InstrumentSchema {
        InstrumentSchema {
            id: Uuid::nil(), tenant_id: Uuid::nil(),
            instrument_type: "CRYPTO".into(), enricher_name: self.name().into(),
            version: "1.0.0".into(), active: true, created_at: chrono::Utc::now(),
            schema_data: serde_json::json!({"inputs":["quantity.traded","price.clean","counterparty","trade_date"]}),
        }
    }
    fn validate(&self, tx: &RawTransaction) -> Vec<ValidationError> { vec![] }
    async fn enrich(&self, tx: &RawTransaction) -> Result<AttributeBag, FbError> {
        let mut attrs = AttributeBag::new();
        let qty = tx.attributes.get_quantity(QuantityType::Traded).unwrap_or(dec!(0));
        let price = tx.attributes.get_price(PriceType::Clean).unwrap_or(dec!(0));
        let gross = qty * price;
        let fee = gross * dec!(0.0015); // 15 bps
        let net = gross + fee;
        attrs.set_amount(AmountType::Gross, gross);
        attrs.set_amount(AmountType::Fee, fee);
        attrs.set_amount(AmountType::Net, net);
        attrs.set_amount(AmountType::Settlement, gross);
        attrs.set_quantity(QuantityType::Current, qty);
        attrs.set_string("currency", "USD");
        Ok(attrs)
    }
    fn posting_rules(&self) -> PostingRules {
        PostingRules {
            template: PostingTemplate {
                name: "crypto_buy".into(), version: "1.0.0".into(),
                entries: vec![
                    PostingRule { side: Side::Debit, account_code_template: "TRADING-{desk}-{instrument_id}".into(), amount_ref: Some("$derived.amount.settlement".into()), quantity_ref: Some("$input.quantity.traded".into()), currency: Some("USD".into()), date_ref: None, narrative: None, subledger_type: Some("TRADING".into()) },
                    PostingRule { side: Side::Debit, account_code_template: "PNL-fee-{desk}".into(), amount_ref: Some("$derived.amount.fee".into()), quantity_ref: None, currency: Some("USD".into()), date_ref: None, narrative: None, subledger_type: Some("PNL".into()) },
                    PostingRule { side: Side::Credit, account_code_template: "CASH-{desk}-USD".into(), amount_ref: Some("$derived.amount.net".into()), quantity_ref: None, currency: Some("USD".into()), date_ref: None, narrative: None, subledger_type: Some("CASH".into()) },
                ],
            },
            link_rules: vec![],
        }
    }
    async fn on_post(&self, _tx: &PostedTransaction) -> Result<(), FbError> { Ok(()) }
}
