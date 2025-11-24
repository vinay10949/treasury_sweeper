//! Scheduler
//!
//! Orchestrates sweep cycles, either once or continuously on a schedule.

use crate::monitor::WalletMonitor;
use crate::types::Config;
use anyhow::Result;
use std::sync::Arc;
use tokio::time::{Duration, sleep};
use tracing::info;

/// Scheduler for orchestrating sweep cycles
pub struct Scheduler {
    monitor: Arc<WalletMonitor>,
    config: Config,
}

impl Scheduler {
    pub fn new(monitor: Arc<WalletMonitor>, config: Config) -> Self {
        Self { monitor, config }
    }


    pub async fn run_once(&self) -> Result<usize> {
        info!("Starting sweep cycle");
        let sweep_count = self.monitor.check_all_wallets(&self.config).await?;
        info!("Sweep cycle complete: {} sweeps executed", sweep_count);

        Ok(sweep_count)
    }

    pub async fn run_continuous(&self) -> Result<()> {
        let interval = Duration::from_secs(self.config.sweep_interval_seconds);

        info!(
            "Starting continuous sweep mode (interval: {}s)",
            self.config.sweep_interval_seconds
        );

        loop {
            match self.run_once().await {
                Ok(count) => {
                    info!("Executed {} sweeps", count);
                }
                Err(e) => {
                    tracing::error!("Error in sweep cycle: {}", e);
                }
            }

            info!(
                "Waiting {} seconds until next cycle...",
                self.config.sweep_interval_seconds
            );
            sleep(interval).await;
        }
    }
}
