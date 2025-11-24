//! Treasury Sweeper
mod balance_checker;
mod monitor;
mod rules_engine;
mod scheduler;
mod state_manager;
mod tx_emitter;
mod types;
use crate::balance_checker::DummyBalanceChecker;
use crate::monitor::*;
use crate::rules_engine::RulesEngine;
use crate::scheduler::*;
use crate::state_manager::StateManager;
use crate::tx_emitter::MockTxEmitter;
use crate::types::*;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::signal;
use tracing::info;
use tracing_subscriber::prelude::*;

#[derive(Parser)]
#[command(name = "treasury_sweeper")]
struct Cli {
    #[arg(short, long, default_value = "config.json")]
    config: PathBuf,

    #[arg(short, long, default_value = "state.json")]
    state: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Once,

    Continuous,

    InitState {
        #[arg(long, default_value = "3")]
        num_wallets: usize,

        #[arg(long, default_value = "60")]
        interval: u64,

        /// ETH threshold for native balance rule
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

    info!("Loading state from {}", cli.state.display());
    let state_manager = Arc::new(StateManager::load(cli.state.clone()).await?);

    // Handle init-state command separately (generates config and state)
    if let Commands::InitState {
        num_wallets,
        interval,
        eth_threshold,
    } = &cli.command
    {
        info!("Initializing configuration and state...");
        let treasury_address = generate_eth_address();
        info!("Generating {} random hot wallet addresses", num_wallets);
        let wallet_addresses: Vec<String> =
            (0..*num_wallets).map(|_| generate_eth_address()).collect();

        info!("Configuration:");
        info!("  Treasury: {}", treasury_address);
        info!("  Hot wallets: {}", wallet_addresses.len());
        info!("  Sweep interval: {}s", interval);
        info!("  ETH threshold: {}", eth_threshold);

        info!("Generated addresses:");
        info!("  Treasury: {}", treasury_address);
        for (i, addr) in wallet_addresses.iter().enumerate() {
            info!("  Hot Wallet {}: {}", i + 1, addr);
        }

        // Create hot wallet configurations
        let mut hot_wallets = Vec::new();
        for (i, address) in wallet_addresses.iter().enumerate() {
            let wallet_config = HotWalletConfig {
                address: address.clone(),
                label: format!("Hot Wallet {}", i + 1),
                rules: vec![
                    SweepRule::NativeBalance {
                        threshold: eth_threshold.clone(),
                        asset: "ETH".to_string(),
                    },
                
                    SweepRule::TokenBalance {
                        threshold: "100".to_string(),
                        token_address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
                        asset: "USDC".to_string(),
                    },
                ],
            };
            hot_wallets.push(wallet_config);
        }

        let config = Config {
            treasury_address: treasury_address,
            hot_wallets,
            sweep_interval_seconds: *interval,
        };
        let config_json =
            serde_json::to_string_pretty(&config).context("Failed to serialize configuration")?;

        tokio::fs::write(&cli.config, config_json)
            .await
            .context("Failed to write configuration file")?;

        info!("✓ Created configuration file: {}", cli.config.display());

        for wallet_config in &config.hot_wallets {
            state_manager
                .initialize_wallet(&wallet_config.address)
                .await?;
        }

        let state = state_manager.fetch_snapshot().await;
        info!("Initialized {} wallets with nonce=0:", state.wallets.len());
        return Ok(());
    }


    info!("Loading configuration from {}", cli.config.display());
    let config_content = tokio::fs::read_to_string(&cli.config)
        .await
        .context("Failed to read config file")?;

    let config: Config =
        serde_json::from_str(&config_content).context("Failed to parse config file")?;

    info!("Configuration loaded:");
    info!(
        "  Treasury: {}",
        config.treasury_address
    );
    info!("  Hot wallets: {}", config.hot_wallets.len());
    info!("  Sweep interval: {}s", config.sweep_interval_seconds);

   
    let balance_checker = DummyBalanceChecker::new(0.0, 4.0);
    let rules_engine = Arc::new(RulesEngine::new(balance_checker));

    let tx_emitter = Arc::new(MockTxEmitter::new(
        state_manager.clone(),
        config.treasury_address.clone(),
    ));

    let monitor = Arc::new(WalletMonitor::new(rules_engine, tx_emitter));
    let scheduler = Scheduler::new(monitor, config);

    // Execute sweep command
    match cli.command {
        Commands::Once => {
            let count = scheduler.run_once().await?;
            info!("Sweep cycle complete: {} sweeps executed", count);
        }
        Commands::Continuous => {
            let ctrl_c = signal::ctrl_c();

            tokio::select! {
                result = scheduler.run_continuous() => {
                    result?;
                }
                _ = ctrl_c => {
                    info!("Received Ctrl+C, shutting down gracefully...");
                }
            }
        }
        Commands::InitState { .. } => {
            unreachable!("InitState handled above");
        }
    }

    info!("Treasury Sweeper shutdown complete");
    Ok(())
}
