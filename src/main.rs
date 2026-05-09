use clap::Parser;

mod kernel;
mod cli;
mod config;
mod db;
mod enrichment;
mod ledger;
mod tenant;

use tracing_subscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("fb=info".parse().unwrap()))
        .init();

    let cli = cli::Cli::parse();
    if let Err(e) = cli::run(cli).await {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}
