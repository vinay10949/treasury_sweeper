//! Wallet Monitor
//!
//! Orchestrates the sweep process: checks balances, evaluates rules,and triggers sweeps when conditions are met.
use crate::rules_engine::RulesEngine;
use crate::tx_emitter::MockTxEmitter;
use crate::types::{Config, HotWalletConfig};
use anyhow::Result;
use std::sync::Arc;
use tracing::{info, warn};

/// Wallet monitor that orchestrates the sweep process
pub struct WalletMonitor {
    rules_engine: Arc<RulesEngine>,
    tx_emitter: Arc<MockTxEmitter>,
}

impl WalletMonitor {
    pub fn new(rules_engine: Arc<RulesEngine>, tx_emitter: Arc<MockTxEmitter>) -> Self {
        Self {
            rules_engine,
            tx_emitter,
        }
    }

    async fn check_and_sweep(&self, wallet_config: &HotWalletConfig) -> Result<usize> {
        info!(
            "Checking wallet {} ({})",
            wallet_config.address,
            wallet_config.label
        );

        // Evaluate all rules
        let decisions = self.rules_engine.evaluate(wallet_config).await?;

        let sweep_count = decisions.len();
        
        // Execute all triggered sweeps
        for decision in decisions {
            self.tx_emitter
                .emit_sweep(&wallet_config.address, &decision)
                .await?;
        }

        Ok(sweep_count)
    }


    /// Returns the total number of sweeps executed across all wallets
    pub async fn check_all_wallets(&self, config: &Config) -> Result<usize> {
        let mut total_sweep_count = 0;

        for wallet_config in &config.hot_wallets {
            match self.check_and_sweep(wallet_config).await {
                Ok(count) => {
                    total_sweep_count += count;
                }
                Err(e) => {
                    warn!(
                        "Error checking wallet {}: {}",
                        wallet_config.address,
                        e
                    );
                    // Continue with other wallets even if one fails
                }
            }
        }

        Ok(total_sweep_count)
    }
}
