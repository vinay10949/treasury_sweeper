//! State Manager - Critical Component for Nonce Management
//!
//! This module implements atomic nonce management with persistent state.

use crate::types::{Address, ServiceState, WalletState};
use anyhow::{Context, Result};
use dashmap::DashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info};

/// State manager with atomic nonce operations
pub struct StateManager {
    state: Arc<RwLock<ServiceState>>,
    ///Used GPT for below pattern.
    /// Per-wallet locks for atomic nonce operations
    /// Without this: All wallet operations would be serialized (slow!)
    /// With this: Only same-wallet operations are serialized (fast concurrent processing)
    wallet_locks: Arc<DashMap<Address, Arc<Mutex<()>>>>,

    state_file_path: PathBuf,
}

impl StateManager {
   
    /// Loads state from the disk
    pub async fn load(state_file_path: PathBuf) -> Result<Self> {

        let state = if state_file_path.exists() {
            info!("Loading state from {}", state_file_path.display());
            let content = fs::read_to_string(&state_file_path)
                .await
                .context("Failed to read state file")?;

            serde_json::from_str::<ServiceState>(&content)
                .context("Failed to parse state file.")?
        } else {
            info!("No state found, creating new state.");
            ServiceState::new()
        };

        for (addr, wallet_state) in &state.wallets {
            debug!(
                "Wallet {} , next_nonce={}, total_sweeps={}",   
                addr, wallet_state.next_nonce, wallet_state.total_sweeps
            );
        }

        Ok(Self {
            state: Arc::new(RwLock::new(state)),
            wallet_locks: Arc::new(DashMap::new()),
            state_file_path,
        })
    }

    /// Atomically reserve and increment nonce for a wallet
    pub async fn reserve_nonce(&self, address: &Address) -> Result<u64> {
        // Get or create per-wallet lock
        let lock = self
            .wallet_locks
            .entry(address.clone())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone();

        let _guard = lock.lock().await;
        let mut state = self.state.write().await;

        if !state.wallets.contains_key(address) {
            info!("Initializing state for new wallet {}", address);
            state
                .wallets
                .insert(address.clone(), WalletState::new(address.clone()));
        }

        let wallet_state = state
            .wallets
            .get_mut(address)
            .expect("Wallet state must exist after initialization");

        let current_nonce = wallet_state.next_nonce;

        wallet_state.next_nonce = wallet_state.next_nonce+1;
        wallet_state.total_sweeps += 1;
        wallet_state.last_sweep_timestamp = Some(chrono::Utc::now().to_rfc3339());
        state.last_update = chrono::Utc::now().to_rfc3339();
        
        self.persist_locked(&state)
            .await
            .context("Failed to persist state after nonce reservation")?;

        Ok(current_nonce)
    }


    /// setup new state for a wallet
    pub async fn initialize_wallet(&self, address: &Address) -> Result<()> {
        let mut state = self.state.write().await;

        if state.wallets.contains_key(address) {
            info!("Wallet {} already initialized", address);
            return Ok(());
        }

        info!("Initializing wallet {}", address);

        state
            .wallets
            .insert(address.clone(), WalletState::new(address.clone()));

        state.last_update = chrono::Utc::now().to_rfc2822();

        self.persist_locked(&state).await?;

        Ok(())
    }

    /// Store the state to a disk.
    async fn persist_locked(&self, state: &ServiceState) -> Result<()> {
        let json = serde_json::to_string_pretty(state).context("Failed to serialize state")?;

        let temp_path = self.state_file_path.with_extension("json.tmp");

        fs::write(&temp_path, &json)
            .await
            .context("Failed to write temporary state file")?;

        let file = fs::File::open(&temp_path).await?;
        file.sync_all()
            .await
            .context("Failed to fsync state file")?;

        fs::rename(&temp_path, &self.state_file_path)
            .await
            .context("Failed to rename state file")?;

        debug!("State persisted to {}", self.state_file_path.display());

        Ok(())
    }

    pub async fn fetch_snapshot(&self) -> ServiceState {
        self.state.read().await.clone()
    }
}
