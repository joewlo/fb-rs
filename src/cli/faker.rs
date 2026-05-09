use std::sync::Arc;
use rand::Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;
use crate::kernel::*;
use crate::ledger::account_store::AccountStore;
use crate::enrichment::*;
use crate::tenant::{PgTenantService, TenantService};
use crate::config::DatabaseConfig;

pub struct Faker {
    pub tenant_code: String,
}

impl Faker {
    pub async fn run(&self, pool: &sqlx::PgPool, count: usize) -> Result<(), FbError> {
        let tenant_svc = PgTenantService::new(pool.clone());

        // Create tenant
        let tenant = match tenant_svc.get_by_code(&self.tenant_code).await {
            Ok(t) => t,
            Err(_) => tenant_svc.create("JP Morgan AM", &self.tenant_code).await?,
        };

        // Build registry
        let mut registry = InMemoryRegistry::new();
        registry.register_contract("BOND", Arc::new(bond::BondContract));
        registry.register_contract("EQUITY", Arc::new(equity::EquityContract));
        registry.register_contract("CRYPTO", Arc::new(crypto::CryptoContract));

        // Build template engine with account resolver
        let account_store = Arc::new(AccountStore::new(pool.clone()));
        let resolver: Box<dyn AccountResolver> = Box::new(PgAccountResolver { store: account_store.clone() });
        let templates = Arc::new(InMemoryTemplateEngine::new(resolver));

        // Build engine
        // Use DB-backed entry writer with batching
        let batch_writer = Arc::new(PgEntryWriter::new(pool.clone()));
        let entry_writer: Arc<dyn EntryWriter + Send + Sync> = batch_writer.clone();
        let registry: Box<dyn ContractRegistry> = Box::new(registry);
        let engine = PostingEngineImpl {
            registry,
            templates,
            entry_writer: entry_writer,
            fee_engine: None,
            position_tracker: None,
            compliance_checker: None,
            event_store: None,
            event_pub: None,
        };

        // Seed accounts
        self.seed_accounts(&account_store, tenant.id).await?;

        // Instruments to trade
        let instruments = vec![
            ("US037833AK99", "BOND", dec!(0.9825)),
            ("US91282CFW45", "BOND", dec!(1.0210)),
            ("IBM.GL", "BOND", dec!(1.0450)),
            ("AAPL", "EQUITY", dec!(185.50)),
            ("MSFT", "EQUITY", dec!(420.30)),
            ("GOOGL", "EQUITY", dec!(175.20)),
            ("JPM", "EQUITY", dec!(198.75)),
            ("GS", "EQUITY", dec!(465.30)),
            ("TSLA", "EQUITY", dec!(250.10)),
            ("NVDA", "EQUITY", dec!(880.45)),
            ("SPY", "EQUITY", dec!(525.60)),
            ("BTC", "CRYPTO", dec!(67250.0)),
            ("ETH", "CRYPTO", dec!(3450.0)),
            ("DOGE", "CRYPTO", dec!(0.15)),
            ("SOL", "CRYPTO", dec!(145.0)),
        ];

        let desks = vec!["ny-fi", "ny-eq", "ny-crypto"];
        let counterparties = vec!["GSCO", "BNPP", "MSCO", "JPM", "DBAG", "CITI", "BOFA", "UBS", "COIN", "KRAK"];
        let mut rng = rand::thread_rng();

        println!("Posting {} transactions...", count);
        let mut success = 0;

        for i in 0..count {
            let (inst_id, inst_type, base_price) = &instruments[rng.gen_range(0..instruments.len())];
            let desk = match *inst_type {
                "BOND" => "ny-fi",
                "EQUITY" => "ny-eq",
                _ => "ny-crypto",
            };
            let cp = counterparties[rng.gen_range(0..counterparties.len())];

            let qty = match *inst_type {
                "BOND" => Decimal::from(rng.gen_range(1_000_000..50_000_000)),
                "EQUITY" => Decimal::from(rng.gen_range(100..500_000)),
                _ => Decimal::from(rng.gen_range(1..100)),
            };

            let price_variation = dec!(0.95) + Decimal::from(rng.gen_range(0..10)) / dec!(100);
            let price = *base_price * price_variation;

            let trade_date = chrono::Utc::now().date_naive() - chrono::Duration::days(rng.gen_range(0..30));

            let mut attrs = AttributeBag::new();
            attrs.set_quantity(QuantityType::Traded, qty);
            attrs.set_price(PriceType::Clean, price.round_dp(4));
            attrs.set_date(DateType::Trade, trade_date);
            attrs.set_string("counterparty", cp);
            attrs.set_string("desk", desk);
            attrs.set_string("currency", "USD");
            attrs.set_string("geo", "us-east");

            let raw = RawTransaction {
                tenant_id: tenant.id,
                instrument_type: (*inst_type).to_string(),
                instrument_id: inst_id.to_string(),
                parent_tx_id: None,
                root_tx_id: None,
                link_type: None,
                link_depth: 0,
                attributes: attrs,
                idempotency_key: Some(Uuid::new_v4()),
                metadata: None,
            };

            match engine.submit(raw).await {
                Ok(_) => success += 1,
                Err(_) => continue,
            }

            if success % 500 == 0 && success > 0 {
                println!("  {}/{} posted", success, count);
            }
        }

        println!("  {}/{} posted successfully", success, count);

        // Flush remaining buffered entries
        batch_writer.flush().await?;

        Ok(())
    }

    async fn seed_accounts(&self, store: &AccountStore, tenant_id: Uuid) -> Result<(), FbError> {
        let accounts = vec![
            ("CASH-ny-fi-USD", "Operating Cash", "ASSET", "CASH", "USD"),
            ("CASH-ny-eq-USD", "Operating Cash", "ASSET", "CASH", "USD"),
            ("CASH-ny-crypto-USD", "Operating Cash", "ASSET", "CASH", "USD"),
            ("TRADING-ny-fi-US037833AK99", "Trading", "ASSET", "TRADING", "USD"),
            ("TRADING-ny-fi-US91282CFW45", "Trading", "ASSET", "TRADING", "USD"),
            ("TRADING-ny-fi-IBM.GL", "Trading", "ASSET", "TRADING", "USD"),
            ("TRADING-ny-eq-AAPL", "Trading", "ASSET", "TRADING", "USD"),
            ("TRADING-ny-eq-MSFT", "Trading", "ASSET", "TRADING", "USD"),
            ("TRADING-ny-eq-GOOGL", "Trading", "ASSET", "TRADING", "USD"),
            ("TRADING-ny-eq-JPM", "Trading", "ASSET", "TRADING", "USD"),
            ("TRADING-ny-eq-GS", "Trading", "ASSET", "TRADING", "USD"),
            ("TRADING-ny-eq-TSLA", "Trading", "ASSET", "TRADING", "USD"),
            ("TRADING-ny-eq-NVDA", "Trading", "ASSET", "TRADING", "USD"),
            ("TRADING-ny-eq-SPY", "Trading", "ASSET", "TRADING", "USD"),
            ("TRADING-ny-crypto-BTC", "Trading", "ASSET", "TRADING", "USD"),
            ("TRADING-ny-crypto-ETH", "Trading", "ASSET", "TRADING", "USD"),
            ("TRADING-ny-crypto-DOGE", "Trading", "ASSET", "TRADING", "USD"),
            ("TRADING-ny-crypto-SOL", "Trading", "ASSET", "TRADING", "USD"),
            ("PNL-commission-ny-fi", "Commission", "EXPENSE", "PNL", "USD"),
            ("PNL-commission-ny-eq", "Commission", "EXPENSE", "PNL", "USD"),
            ("PNL-commission-ny-crypto", "Commission", "EXPENSE", "PNL", "USD"),
            ("PNL-stamp_duty-ny-eq", "Stamp Duty", "EXPENSE", "PNL", "USD"),
            ("PNL-fee-ny-crypto", "Fees", "EXPENSE", "PNL", "USD"),
            ("PNL-fee-ny-fi", "Fees", "EXPENSE", "PNL", "USD"),
            ("PNL-accrued_interest-ny-fi", "Accrued Interest", "INCOME", "PNL", "USD"),
        ];

        for (code, name, atype_str, sub, ccy) in &accounts {
            let atype = match *atype_str {
                "ASSET" => AccountType::Asset,
                "LIABILITY" => AccountType::Liability,
                "EQUITY" => AccountType::Equity,
                "INCOME" => AccountType::Income,
                "EXPENSE" => AccountType::Expense,
                _ => AccountType::Asset,
            };
            let subledger = match *sub {
                "CASH" => Some(SubledgerType::Cash),
                "TRADING" => Some(SubledgerType::Trading),
                "PNL" => Some(SubledgerType::PNL),
                "SETTLEMENT" => Some(SubledgerType::Settlement),
                "POSITION" => Some(SubledgerType::Position),
                _ => None,
            };
            let _ = store.create_account(tenant_id, "us-east", code, name, atype, subledger, ccy, None, None, None, None).await;
        }
        Ok(())
    }
}

struct PgAccountResolver {
    store: Arc<AccountStore>,
}

#[async_trait::async_trait]
impl AccountResolver for PgAccountResolver {
    async fn resolve(&self, code_template: &str, ctx: &ResolveContext) -> Result<Uuid, FbError> {
        self.store.resolve(code_template, ctx).await
    }
}
