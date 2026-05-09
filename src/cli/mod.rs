pub mod output;
pub mod faker;

use anyhow::Result;
use clap::{Parser, Subcommand};
use output::OutputFormat;

/// Financial Back Office System.
#[derive(Parser, Debug)]
#[command(name = "fb", version, about, long_about = None)]
pub struct Cli {
    /// Path to configuration file.
    #[arg(long, global = true, default_value = "config.yaml")]
    pub config: String,

    /// Tenant identifier.
    #[arg(short = 't', long, global = true)]
    pub tenant: Option<String>,

    /// Output format.
    #[arg(long, global = true, default_value = "table")]
    pub format: OutputFormat,

    /// Enable verbose output.
    #[arg(short = 'v', long, global = true, default_value_t = false)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Manage tenants.
    Tenant {
        #[command(subcommand)]
        action: TenantAction,
    },
    /// Work with transactions.
    Tx {
        #[command(subcommand)]
        action: TxAction,
    },
    /// Book-level operations.
    Book {
        #[command(subcommand)]
        action: BookAction,
    },
    /// Ledger queries.
    Ledger {
        #[command(subcommand)]
        action: LedgerAction,
    },
    /// Chart of accounts management.
    Coa {
        #[command(subcommand)]
        action: CoaAction,
    },
    /// Schema management.
    Schema {
        #[command(subcommand)]
        action: SchemaAction,
    },
    /// Smart contract lifecycle.
    Contract {
        #[command(subcommand)]
        action: ContractAction,
    },
    /// Server operations.
    Server {
        #[command(subcommand)]
        action: ServerAction,
    },
    /// Agent operations.
    Agent {
        #[command(subcommand)]
        action: AgentAction,
    },
    /// Event sourcing operations.
    Event {
        #[command(subcommand)]
        action: EventAction,
    },
    /// Generate fake data.
    Faker {
        /// Number of records to generate.
        #[arg(short = 'n')]
        count: u64,
    },
    /// Tax calculations.
    Tax {
        #[command(subcommand)]
        action: TaxAction,
    },
    /// Performance metrics.
    Perf {
        #[command(subcommand)]
        action: PerfAction,
    },
    /// Statement generation.
    Statement {
        #[command(subcommand)]
        action: StatementAction,
    },
    /// Compliance operations.
    Compliance {
        #[command(subcommand)]
        action: ComplianceAction,
    },
}

// ─── Tenant ──────────────────────────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum TenantAction {
    /// Create a new tenant.
    Create {
        /// Tenant name.
        name: String,
    },
    /// List all tenants.
    List,
    /// Activate a tenant.
    Activate {
        /// Tenant ID.
        id: String,
    },
    /// Deactivate a tenant.
    Deactivate {
        /// Tenant ID.
        id: String,
    },
    /// Describe a tenant.
    Describe {
        /// Tenant ID.
        id: String,
    },
}

// ─── Transaction ─────────────────────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum TxAction {
    /// Post a new transaction.
    Post {
        /// Tenant.
        tenant: String,
        /// Amount.
        amount: String,
        /// Currency code (ISO 4217).
        currency: String,
        /// Debit account.
        #[arg(long)]
        debit_account: String,
        /// Credit account.
        #[arg(long)]
        credit_account: String,
        /// Optional description.
        #[arg(long)]
        description: Option<String>,
    },
    /// Get transaction status.
    Status {
        /// Transaction ID.
        id: String,
    },
    /// List transactions.
    List {
        /// Show only pending transactions.
        #[arg(long)]
        pending: bool,
        /// Maximum number of results.
        #[arg(long, default_value = "50")]
        limit: u64,
    },
    /// List child transactions.
    Children {
        /// Parent transaction ID.
        id: String,
    },
    /// Show the transaction DAG.
    Dag {
        /// Root transaction ID.
        id: String,
    },
    /// Cancel a pending transaction.
    Cancel {
        /// Transaction ID.
        id: String,
    },
}

// ─── Book ─────────────────────────────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum BookAction {
    /// View a book.
    View {
        /// Book name.
        name: String,
    },
    /// Show book position.
    Position {
        /// Book name.
        name: String,
    },
    /// Profit & Loss operations.
    Pnl {
        #[command(subcommand)]
        action: PnlAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum PnlAction {
    /// Daily P&L view.
    Daily {
        /// Book name (optional).
        #[arg(long)]
        book: Option<String>,
        /// Reference date (YYYY-MM-DD).
        #[arg(long)]
        date: Option<String>,
    },
    /// Summary P&L view.
    Summary {
        /// Book name (optional).
        #[arg(long)]
        book: Option<String>,
        /// Start date (YYYY-MM-DD).
        #[arg(long)]
        start: Option<String>,
        /// End date (YYYY-MM-DD).
        #[arg(long)]
        end: Option<String>,
    },
}

// ─── Ledger ───────────────────────────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum LedgerAction {
    /// Trial balance report.
    TrialBalance {
        /// Reference date (YYYY-MM-DD).
        #[arg(long)]
        date: Option<String>,
        /// Tenant filter.
        #[arg(long)]
        tenant: Option<String>,
    },
    /// Account detail view.
    AccountDetail {
        /// Account identifier.
        account: String,
        /// Reference date (YYYY-MM-DD).
        #[arg(long)]
        date: Option<String>,
        /// Tenant filter.
        #[arg(long)]
        tenant: Option<String>,
    },
    /// Subledger queries.
    Subledger {
        #[command(subcommand)]
        action: SubledgerAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum SubledgerAction {
    /// Trading subledger.
    Trading {
        /// Reference date (YYYY-MM-DD).
        #[arg(long)]
        date: Option<String>,
        /// Tenant filter.
        #[arg(long)]
        tenant: Option<String>,
    },
    /// Cash subledger.
    Cash {
        /// Reference date (YYYY-MM-DD).
        #[arg(long)]
        date: Option<String>,
        /// Tenant filter.
        #[arg(long)]
        tenant: Option<String>,
    },
    /// P&L subledger.
    Pnl {
        /// Reference date (YYYY-MM-DD).
        #[arg(long)]
        date: Option<String>,
        /// Tenant filter.
        #[arg(long)]
        tenant: Option<String>,
    },
    /// Settlement subledger.
    Settlement {
        /// Reference date (YYYY-MM-DD).
        #[arg(long)]
        date: Option<String>,
        /// Tenant filter.
        #[arg(long)]
        tenant: Option<String>,
    },
}

// ─── Chart of Accounts ────────────────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum CoaAction {
    /// Import COA from a file.
    Import {
        /// Path to COA file.
        file: String,
    },
    /// Create a new account.
    Create {
        /// Account name.
        name: String,
        /// Account type.
        #[arg(long)]
        account_type: String,
    },
    /// List all accounts.
    List,
    /// Describe an account.
    Describe {
        /// Account ID.
        id: String,
    },
}

// ─── Schema ───────────────────────────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum SchemaAction {
    /// Register a new schema.
    Register {
        /// Schema name.
        name: String,
        /// Schema version.
        version: String,
    },
    /// List registered schemas.
    List,
    /// Describe a schema.
    Describe {
        /// Schema ID.
        id: String,
    },
}

// ─── Contract ─────────────────────────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum ContractAction {
    /// Register a new contract.
    Register {
        /// Contract name.
        name: String,
        /// Contract source code or path.
        code: String,
    },
    /// List registered contracts.
    List,
    /// Describe a contract.
    Describe {
        /// Contract ID.
        id: String,
    },
    /// Run contract tests.
    Test {
        /// Contract ID.
        id: String,
    },
    /// Deploy a contract.
    Deploy {
        /// Contract ID.
        id: String,
    },
    /// Rollback a contract deployment.
    Rollback {
        /// Contract ID.
        id: String,
    },
}

// ─── Server ───────────────────────────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum ServerAction {
    /// Start the server.
    Start {
        /// Listen port.
        #[arg(long, default_value = "8080")]
        port: u16,
    },
    /// Run database migrations.
    Migrate {
        #[command(subcommand)]
        direction: MigrateDirection,
    },
    /// Health check against running server.
    Health,
}

#[derive(Subcommand, Debug)]
pub enum MigrateDirection {
    /// Apply pending migrations.
    Up,
    /// Roll back the last migration.
    Down,
}

// ─── Agent ────────────────────────────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum AgentAction {
    /// Run an agent task.
    Run {
        /// Task name or ID.
        task: String,
    },
    /// Interactive chat with the agent.
    Chat {
        /// Chat message.
        message: String,
    },
}

// ─── Event ────────────────────────────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum EventAction {
    /// Replay events for a given ID.
    Replay {
        /// Event or aggregate ID to replay.
        #[arg(long)]
        event_id: Option<String>,
    },
    /// Reconcile the event store with projections.
    Reconcile {
        /// Start offset or timestamp.
        #[arg(long)]
        from: Option<String>,
        /// End offset or timestamp.
        #[arg(long)]
        to: Option<String>,
    },
}

// ─── Tax ──────────────────────────────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum TaxAction {
    /// Calculate tax liability.
    Calculate {
        /// Tenant filter.
        #[arg(long)]
        tenant: Option<String>,
        /// Tax year.
        #[arg(long)]
        year: Option<u16>,
    },
}

// ─── Performance ──────────────────────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum PerfAction {
    /// Compute internal rate of return.
    Irr {
        /// Investment ID.
        #[arg(long)]
        investment_id: Option<String>,
        /// File with cash flow records (one per line).
        #[arg(long)]
        cash_flows: Option<String>,
    },
}

// ─── Statement ────────────────────────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum StatementAction {
    /// Position statement.
    Position {
        /// Tenant filter.
        #[arg(long)]
        tenant: Option<String>,
        /// Reference date (YYYY-MM-DD).
        #[arg(long)]
        date: Option<String>,
    },
    /// Transaction statement.
    Transactions {
        /// Tenant filter.
        #[arg(long)]
        tenant: Option<String>,
        /// Start date (YYYY-MM-DD).
        #[arg(long)]
        from: Option<String>,
        /// End date (YYYY-MM-DD).
        #[arg(long)]
        to: Option<String>,
    },
}

// ─── Compliance ───────────────────────────────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum ComplianceAction {
    /// Run compliance checks.
    Check {
        /// Tenant filter.
        #[arg(long)]
        tenant: Option<String>,
        /// Reference date (YYYY-MM-DD).
        #[arg(long)]
        date: Option<String>,
    },
    /// List compliance alerts.
    Alerts {
        /// Tenant filter.
        #[arg(long)]
        tenant: Option<String>,
    },
    /// Seed compliance rules.
    Seed,
}

// ─── Dispatcher ───────────────────────────────────────────────────────────────

/// Route a parsed CLI to its handler.
pub async fn run(cli: Cli) -> Result<()> {
    if cli.verbose {
        eprintln!("fb: config={} tenant={:?} format={:?}", cli.config, cli.tenant, cli.format);
    }

    match cli.command {
        Command::Tenant { action } => handle_tenant(action).await,
        Command::Tx { action } => handle_tx(action).await,
        Command::Book { action } => handle_book(action).await,
        Command::Ledger { action } => handle_ledger(action).await,
        Command::Coa { action } => handle_coa(action).await,
        Command::Schema { action } => handle_schema(action).await,
        Command::Contract { action } => handle_contract(action).await,
        Command::Server { action } => handle_server(action).await,
        Command::Agent { action } => handle_agent(action).await,
        Command::Event { action } => handle_event(action).await,
        Command::Faker { count } => handle_faker(count).await,
        Command::Tax { action } => handle_tax(action).await,
        Command::Perf { action } => handle_perf(action).await,
        Command::Statement { action } => handle_statement(action).await,
        Command::Compliance { action } => handle_compliance(action).await,
    }
}

// ─── Handlers ─────────────────────────────────────────────────────────────────

async fn handle_tenant(action: TenantAction) -> Result<()> {
    match action {
        TenantAction::Create { name } => not_implemented(&format!("tenant create {}", name)),
        TenantAction::List => not_implemented("tenant list"),
        TenantAction::Activate { id } => not_implemented(&format!("tenant activate {}", id)),
        TenantAction::Deactivate { id } => not_implemented(&format!("tenant deactivate {}", id)),
        TenantAction::Describe { id } => not_implemented(&format!("tenant describe {}", id)),
    }
}

async fn handle_tx(action: TxAction) -> Result<()> {
    match action {
        TxAction::Post { tenant, amount, currency, .. } => {
            not_implemented(&format!("tx post {} {} {}", tenant, amount, currency))
        }
        TxAction::Status { id } => not_implemented(&format!("tx status {}", id)),
        TxAction::List { pending, limit } => {
            not_implemented(&format!("tx list pending={} limit={}", pending, limit))
        }
        TxAction::Children { id } => not_implemented(&format!("tx children {}", id)),
        TxAction::Dag { id } => not_implemented(&format!("tx dag {}", id)),
        TxAction::Cancel { id } => not_implemented(&format!("tx cancel {}", id)),
    }
}

async fn handle_book(action: BookAction) -> Result<()> {
    match action {
        BookAction::View { name } => not_implemented(&format!("book view {}", name)),
        BookAction::Position { name } => not_implemented(&format!("book position {}", name)),
        BookAction::Pnl { action } => match action {
            PnlAction::Daily { book, date } => {
                not_implemented(&format!("book pnl daily book={:?} date={:?}", book, date))
            }
            PnlAction::Summary { book, start, end } => {
                not_implemented(&format!(
                    "book pnl summary book={:?} start={:?} end={:?}",
                    book, start, end
                ))
            }
        },
    }
}

async fn handle_ledger(action: LedgerAction) -> Result<()> {
    match action {
        LedgerAction::TrialBalance { date, tenant } => {
            not_implemented(&format!("ledger trial-balance date={:?} tenant={:?}", date, tenant))
        }
        LedgerAction::AccountDetail {
            account,
            date,
            tenant,
        } => not_implemented(&format!(
            "ledger account-detail {} date={:?} tenant={:?}",
            account, date, tenant
        )),
        LedgerAction::Subledger { action } => match action {
            SubledgerAction::Trading { date, tenant } => not_implemented(&format!(
                "ledger subledger trading date={:?} tenant={:?}",
                date, tenant
            )),
            SubledgerAction::Cash { date, tenant } => {
                not_implemented(&format!("ledger subledger cash date={:?} tenant={:?}", date, tenant))
            }
            SubledgerAction::Pnl { date, tenant } => {
                not_implemented(&format!("ledger subledger pnl date={:?} tenant={:?}", date, tenant))
            }
            SubledgerAction::Settlement { date, tenant } => not_implemented(&format!(
                "ledger subledger settlement date={:?} tenant={:?}",
                date, tenant
            )),
        },
    }
}

async fn handle_coa(action: CoaAction) -> Result<()> {
    match action {
        CoaAction::Import { file } => not_implemented(&format!("coa import {}", file)),
        CoaAction::Create { name, account_type } => {
            not_implemented(&format!("coa create {} {}", name, account_type))
        }
        CoaAction::List => not_implemented("coa list"),
        CoaAction::Describe { id } => not_implemented(&format!("coa describe {}", id)),
    }
}

async fn handle_schema(action: SchemaAction) -> Result<()> {
    match action {
        SchemaAction::Register { name, version } => {
            not_implemented(&format!("schema register {} v{}", name, version))
        }
        SchemaAction::List => not_implemented("schema list"),
        SchemaAction::Describe { id } => not_implemented(&format!("schema describe {}", id)),
    }
}

async fn handle_contract(action: ContractAction) -> Result<()> {
    match action {
        ContractAction::Register { name, code } => {
            not_implemented(&format!("contract register {} {}", name, code))
        }
        ContractAction::List => not_implemented("contract list"),
        ContractAction::Describe { id } => not_implemented(&format!("contract describe {}", id)),
        ContractAction::Test { id } => not_implemented(&format!("contract test {}", id)),
        ContractAction::Deploy { id } => not_implemented(&format!("contract deploy {}", id)),
        ContractAction::Rollback { id } => not_implemented(&format!("contract rollback {}", id)),
    }
}

async fn handle_server(action: ServerAction) -> Result<()> {
    match action {
        ServerAction::Start { port } => not_implemented(&format!("server start :{}", port)),
        ServerAction::Migrate { direction } => match direction {
            MigrateDirection::Up => not_implemented("server migrate up"),
            MigrateDirection::Down => not_implemented("server migrate down"),
        },
        ServerAction::Health => not_implemented("server health"),
    }
}

async fn handle_agent(action: AgentAction) -> Result<()> {
    match action {
        AgentAction::Run { task } => not_implemented(&format!("agent run {}", task)),
        AgentAction::Chat { message } => not_implemented(&format!("agent chat {}", message)),
    }
}

async fn handle_event(action: EventAction) -> Result<()> {
    match action {
        EventAction::Replay { event_id } => {
            not_implemented(&format!("event replay {:?}", event_id))
        }
        EventAction::Reconcile { from, to } => {
            not_implemented(&format!("event reconcile from={:?} to={:?}", from, to))
        }
    }
}

async fn handle_faker(count: u64) -> Result<()> {
    let pool = crate::config::DatabaseConfig::default().create_pool().await
        .map_err(|e| anyhow::anyhow!("db: {}", e))?;

    let f = crate::cli::faker::Faker { tenant_code: "jpmam".into() };
    f.run(&pool, count as usize).await.map_err(|e| anyhow::anyhow!("{}", e))
}

async fn handle_tax(action: TaxAction) -> Result<()> {
    match action {
        TaxAction::Calculate { tenant, year } => {
            not_implemented(&format!("tax calculate tenant={:?} year={:?}", tenant, year))
        }
    }
}

async fn handle_perf(action: PerfAction) -> Result<()> {
    match action {
        PerfAction::Irr {
            investment_id,
            cash_flows,
        } => not_implemented(&format!(
            "perf irr investment_id={:?} cash_flows={:?}",
            investment_id, cash_flows
        )),
    }
}

async fn handle_statement(action: StatementAction) -> Result<()> {
    match action {
        StatementAction::Position { tenant, date } => {
            not_implemented(&format!("statement position tenant={:?} date={:?}", tenant, date))
        }
        StatementAction::Transactions { tenant, from, to } => not_implemented(&format!(
            "statement transactions tenant={:?} from={:?} to={:?}",
            tenant, from, to
        )),
    }
}

async fn handle_compliance(action: ComplianceAction) -> Result<()> {
    match action {
        ComplianceAction::Check { tenant, date } => {
            not_implemented(&format!("compliance check tenant={:?} date={:?}", tenant, date))
        }
        ComplianceAction::Alerts { tenant } => {
            not_implemented(&format!("compliance alerts tenant={:?}", tenant))
        }
        ComplianceAction::Seed => not_implemented("compliance seed"),
    }
}

fn not_implemented(cmd: &str) -> Result<()> {
    println!("not implemented: {}", cmd);
    Ok(())
}
