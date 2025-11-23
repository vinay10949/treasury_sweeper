//! Treasury Sweeper CLI
mod types;
use crate::types::*;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::info;

#[derive(Parser)]
#[command(name = "treasury_sweeper")]
struct Cli {
    /// Config file path
    #[arg(short, long, default_value = "config.json")]
    config: PathBuf,

    /// state file path
    #[arg(short, long, default_value = "state.json")]
    state: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    InitState {
        #[arg(long, default_value = "3")]
        num_wallets: usize,

        #[arg(long, default_value = "60")]
        interval: u64,

        #[arg(long, default_value = "0.1")]
        eth_threshold: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "treasury_sweeper=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();

    if let Commands::InitState {
        num_wallets,
        interval,
        eth_threshold,
    } = &cli.command
    {
        info!("Initializing configuration and state...");

        let addr = types::generate_eth_address();
        info!(
            "Generated treasury address: {}",
            types::truncate_address(&addr)
        );

        info!("Generating {} random hot wallet addresses", num_wallets);

        let wallet_addresses: Vec<String> = (0..*num_wallets)
            .map(|_| types::generate_eth_address())
            .collect();

        info!("Configuration:");
        info!("  Treasury: {}", addr);
        info!("  Hot wallets: {}", wallet_addresses.len());
        info!("  Sweep interval: {}s", interval);
        info!("  ETH threshold: {}", eth_threshold);

        info!("Generated addresses:");
        info!("  Treasury: {}", addr);
        for (i, addr) in wallet_addresses.iter().enumerate() {
            info!("  Hot Wallet {}: {}", i + 1, addr);
        }

        let mut hot_wallets = Vec::new();
        for (i, address) in wallet_addresses.iter().enumerate() {
            let wallet_config = HotWalletConfig {
                address: address.clone(),
                label: format!("Hot Wallet {}", i + 1),
                rules: vec![SweepRule::NativeBalance {
                    threshold: eth_threshold.clone(),
                    asset: "ETH".to_string(),
                }],
            };
            hot_wallets.push(wallet_config);
        }

        let config = Config {
            treasury_address: addr,
            hot_wallets,
            sweep_interval_seconds: *interval,
        };

        let config_json =
            serde_json::to_string_pretty(&config).context("Failed to serialize configuration")?;

        tokio::fs::write(&cli.config, config_json)
            .await
            .context("Failed to write configuration file")?;

        info!("✓ Created configuration file: {}", cli.config.display());

        return Ok(());
    }
    Ok(())
}
