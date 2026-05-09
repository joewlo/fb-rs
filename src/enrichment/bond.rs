use async_trait::async_trait;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;
use crate::kernel::*;

pub struct BondContract;

#[async_trait]
impl Contract for BondContract {
    fn name(&self) -> &str { "bond_trade" }

    fn schema(&self) -> InstrumentSchema {
        InstrumentSchema {
            id: Uuid::nil(), tenant_id: Uuid::nil(),
            instrument_type: "BOND".into(), enricher_name: self.name().into(),
            version: "1.0.0".into(), active: true, created_at: chrono::Utc::now(),
            schema_data: serde_json::json!({"inputs":["quantity.traded","price.clean","counterparty","trade_date"]}),
        }
    }

    fn validate(&self, tx: &RawTransaction) -> Vec<ValidationError> {
        let mut errs = vec![];
        if tx.attributes.get_quantity(QuantityType::Traded).is_none() {
            errs.push(ValidationError { field: "quantity.traded".into(), code: "MISSING".into(), message: "required".into() });
        }
        errs
    }

    async fn enrich(&self, tx: &RawTransaction) -> Result<AttributeBag, FbError> {
        let mut attrs = AttributeBag::new();
        let qty = tx.attributes.get_quantity(QuantityType::Traded).unwrap_or(dec!(0));
        let price = tx.attributes.get_price(PriceType::Clean).unwrap_or(dec!(0));

        let gross = qty * price;
        let commission = gross * dec!(0.0005); // 5 bps
        let net = gross + commission;

        attrs.set_amount(AmountType::Gross, gross);
        attrs.set_amount(AmountType::Commission, commission);
        attrs.set_amount(AmountType::Net, net);
        attrs.set_amount(AmountType::Settlement, gross);
        attrs.set_date(DateType::Settlement, chrono::Utc::now().date_naive() + chrono::Duration::days(2));
        attrs.set_quantity(QuantityType::Current, qty);
        attrs.set_string("currency", "USD");

        Ok(attrs)
    }

    fn posting_rules(&self) -> PostingRules {
        PostingRules {
            template: PostingTemplate {
                name: "bond_buy".into(), version: "1.0.0".into(),
                entries: vec![
                    PostingRule { side: Side::Debit, account_code_template: "TRADING-{desk}-{instrument_id}".into(), amount_ref: Some("$derived.amount.settlement".into()), quantity_ref: Some("$input.quantity.traded".into()), currency: Some("USD".into()), date_ref: None, narrative: None, subledger_type: Some("TRADING".into()) },
                    PostingRule { side: Side::Debit, account_code_template: "PNL-commission-{desk}".into(), amount_ref: Some("$derived.amount.commission".into()), quantity_ref: None, currency: Some("USD".into()), date_ref: None, narrative: None, subledger_type: Some("PNL".into()) },
                    PostingRule { side: Side::Credit, account_code_template: "CASH-{desk}-USD".into(), amount_ref: Some("$derived.amount.net".into()), quantity_ref: None, currency: Some("USD".into()), date_ref: None, narrative: None, subledger_type: Some("CASH".into()) },
                ],
            },
            link_rules: vec![],
        }
    }

    async fn on_post(&self, _tx: &PostedTransaction) -> Result<(), FbError> { Ok(()) }
}
