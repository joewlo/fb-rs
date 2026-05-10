use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;
use crate::kernel::*;
use crate::kernel::fast_writer::FastBatchWriter;
use crate::kernel::cached_resolver::CachedAccountResolver;
use crate::enrichment::*;
use crate::enrichment::registry::InMemoryRegistry;
use crate::tenant::{PgTenantService, TenantService};

static ID_COUNTER: AtomicU64 = AtomicU64::new(1);
fn next_id() -> Uuid { Uuid::from_u64_pair(0, ID_COUNTER.fetch_add(1, Ordering::Relaxed)) }

pub async fn run_benchmark(pool: &sqlx::PgPool, count: usize) -> Result<(), FbError> {
    let total_start = Instant::now();
    let tenant_svc = PgTenantService::new(pool.clone());
    let tenant = match tenant_svc.get_by_code("jpmam").await {
        Ok(t) => t, Err(_) => tenant_svc.create("JP Morgan AM", "jpmam").await?,
    };
    let tenant_id = tenant.id;

    seed_accounts(pool, tenant_id).await?;

    let writer = Arc::new(FastBatchWriter::new(pool.clone()));

    let instruments: Vec<(&str, &str, Decimal)> = vec![
        ("US037833AK99","BOND",dec!(0.9825)), ("AAPL","EQUITY",dec!(185.50)),
        ("MSFT","EQUITY",dec!(420.30)), ("GOOGL","EQUITY",dec!(175.20)),
        ("JPM","EQUITY",dec!(198.75)), ("GS","EQUITY",dec!(465.30)),
        ("TSLA","EQUITY",dec!(250.10)), ("NVDA","EQUITY",dec!(880.45)),
        ("SPY","EQUITY",dec!(525.60)), ("BTC","CRYPTO",dec!(67250.0)),
        ("ETH","CRYPTO",dec!(3450.0)), ("DOGE","CRYPTO",dec!(0.15)),
        ("SOL","CRYPTO",dec!(145.0)), ("IBM.GL","BOND",dec!(1.0450)),
        ("US91282CFW45","BOND",dec!(1.0210)),
    ];
    let instruments = Arc::new(instruments);
    let counterparties = Arc::new(["GSCO","BNPP","MSCO","JPM","DBAG","CITI","BOFA","UBS","COIN","KRAK"]);

    let concurrency = 8;
    let per_task = count / concurrency;
    println!("Bench: {} tx, {} parallel tasks ({} each)", count, concurrency, per_task);

    let mut handles = Vec::new();
    for _ in 0..concurrency {
        let pool = pool.clone();
        let writer = writer.clone();
        let instruments = instruments.clone();
        let counterparties = counterparties.clone();
        handles.push(tokio::spawn(async move {
            let resolver = CachedAccountResolver::new(pool.clone()).await?;
            let templates = Arc::new(InMemoryTemplateEngine::new(Box::new(resolver)));
            let mut registry = InMemoryRegistry::new();
            registry.register_contract("BOND", Arc::new(bond::BondContract));
            registry.register_contract("EQUITY", Arc::new(equity::EquityContract));
            registry.register_contract("CRYPTO", Arc::new(crypto::CryptoContract));
            let eng = PostingEngineImpl {
                registry: Box::new(registry),
                templates,
                entry_writer: writer as Arc<dyn EntryWriter + Send + Sync>,
                fee_engine: None, position_tracker: None, compliance_checker: None,
                event_store: None, event_pub: None,
            };
            let mut rng = StdRng::from_entropy();
            let mut success = 0u64;
            for _ in 0..per_task {
                let idx = rng.gen_range(0..instruments.len());
                let (inst_id, inst_type, base_price) = &instruments[idx];
                let desk = match *inst_type { "BOND" => "ny-fi", "EQUITY" => "ny-eq", _ => "ny-crypto" };
                let cp = counterparties[rng.gen_range(0..counterparties.len())];
                let qty = match *inst_type {
                    "BOND" => Decimal::from(rng.gen_range(1_000_000..50_000_000)),
                    "EQUITY" => Decimal::from(rng.gen_range(100..500_000)),
                    _ => Decimal::from(rng.gen_range(1..100)),
                };
                let price = *base_price * (dec!(0.95) + Decimal::from(rng.gen_range(0..10)) / dec!(100));
                let mut attrs = AttributeBag::new();
                attrs.set_quantity(QuantityType::Traded, qty);
                attrs.set_price(PriceType::Clean, price.round_dp(4));
                attrs.set_date(DateType::Trade, chrono::Utc::now().date_naive() - chrono::Duration::days(rng.gen_range(0..30)));
                attrs.set_string("counterparty", cp);
                attrs.set_string("desk", desk);
                attrs.set_string("currency", "USD"); attrs.set_string("geo", "us-east");
                let raw = RawTransaction {
                    tenant_id, instrument_type: (*inst_type).to_string(), instrument_id: inst_id.to_string(),
                    parent_tx_id: None, root_tx_id: None, link_type: None, link_depth: 0,
                    attributes: attrs, idempotency_key: Some(next_id()), metadata: None,
                };
                if eng.submit(raw).await.is_ok() { success += 1; }
            }
            Ok::<u64, FbError>(success)
        }));
    }

    let mut total_success = 0u64;
    for h in handles {
        total_success += h.await.unwrap().unwrap_or(0);
    }
    let t0 = Instant::now();
    writer.flush().await?;
    let flush_ms = t0.elapsed().as_micros() as f64 / 1000.0;
    let total = total_start.elapsed().as_secs_f64();

    println!("\n=== RESULTS ===");
    println!("  Tx: {} | {:.2}s | {:.0}/s | flush={:.1}ms | tasks={}",
        total_success, total, total_success as f64 / total, flush_ms, concurrency);
    Ok(())
}

async fn seed_accounts(pool: &sqlx::PgPool, tenant_id: Uuid) -> Result<(), FbError> {
    let accounts = [
        ("CASH-ny-fi-USD","ASSET","CASH","USD"), ("CASH-ny-eq-USD","ASSET","CASH","USD"),
        ("CASH-ny-crypto-USD","ASSET","CASH","USD"),
        ("TRADING-ny-fi-US037833AK99","ASSET","TRADING","USD"),
        ("TRADING-ny-fi-US91282CFW45","ASSET","TRADING","USD"),
        ("TRADING-ny-fi-IBM.GL","ASSET","TRADING","USD"),
        ("TRADING-ny-eq-AAPL","ASSET","TRADING","USD"),
        ("TRADING-ny-eq-MSFT","ASSET","TRADING","USD"),
        ("TRADING-ny-eq-GOOGL","ASSET","TRADING","USD"),
        ("TRADING-ny-eq-JPM","ASSET","TRADING","USD"),
        ("TRADING-ny-eq-GS","ASSET","TRADING","USD"),
        ("TRADING-ny-eq-TSLA","ASSET","TRADING","USD"),
        ("TRADING-ny-eq-NVDA","ASSET","TRADING","USD"),
        ("TRADING-ny-eq-SPY","ASSET","TRADING","USD"),
        ("TRADING-ny-crypto-BTC","ASSET","TRADING","USD"),
        ("TRADING-ny-crypto-ETH","ASSET","TRADING","USD"),
        ("TRADING-ny-crypto-DOGE","ASSET","TRADING","USD"),
        ("TRADING-ny-crypto-SOL","ASSET","TRADING","USD"),
        ("PNL-commission-ny-fi","EXPENSE","PNL","USD"),
        ("PNL-commission-ny-eq","EXPENSE","PNL","USD"),
        ("PNL-commission-ny-crypto","EXPENSE","PNL","USD"),
        ("PNL-stamp_duty-ny-eq","EXPENSE","PNL","USD"),
        ("PNL-fee-ny-crypto","EXPENSE","PNL","USD"),
        ("PNL-fee-ny-fi","EXPENSE","PNL","USD"),
        ("PNL-accrued_interest-ny-fi","INCOME","PNL","USD"),
    ];
    for (code, atype, sub, ccy) in &accounts {
        let _ = sqlx::query("INSERT INTO accounts (id,tenant_id,geo,account_code,account_name,account_type,subledger_type,currency,balance,frozen_balance,version,sequence_number,status,created_at,updated_at) VALUES ($1,$2,$3,$4,$4,$5,$6,$7,0,0,0,0,'active',NOW(),NOW()) ON CONFLICT (tenant_id,geo,account_code) DO NOTHING")
            .bind(next_id()).bind(tenant_id).bind("us-east").bind(code).bind(atype).bind(*sub).bind(*ccy)
            .execute(pool).await;
    }
    Ok(())
}
