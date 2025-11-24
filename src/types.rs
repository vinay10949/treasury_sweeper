//! Core data types for the Treasury Sweeper Service
#[allow(unused)]
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type Address = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub treasury_address: Address,
    pub hot_wallets: Vec<HotWalletConfig>,
    pub sweep_interval_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotWalletConfig {
    pub address: Address,
    pub label: String,
    pub rules: Vec<SweepRule>,
}

// Dummy eth address generator
pub fn generate_eth_address() -> String {
    let mut rng = rand::rng();
    let mut bytes = [0u8; 20];
    rng.fill(&mut bytes);

    format!("0x{}", hex::encode(bytes))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SweepRule {
    #[serde(rename = "native_balance")]
    NativeBalance {
        threshold: String,
        asset: String, //eth,sol,dot etc
    },

    #[serde(rename = "token_balance")]
    TokenBalance {
        threshold: String,
        token_address: Address,
        asset: String, //usdc,usdt,dai etc
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceState {
    pub wallets: HashMap<Address, WalletState>,
    pub last_update: String,
}

impl ServiceState {
    pub fn new() -> Self {
        Self {
            wallets: HashMap::new(),
            last_update: chrono::Utc::now().to_rfc3339(),
        }
    }
}

impl Default for ServiceState {
    fn default() -> Self {
        Self::new()
    }
}

/// State for a single wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletState {
    pub address: Address,
    pub next_nonce: u64,
    pub last_sweep_timestamp: Option<String>,
    pub total_sweeps: u64,
}

impl WalletState {
    pub fn new(address: Address) -> Self {
        Self {
            address,
            next_nonce: 0,
            last_sweep_timestamp: None,
            total_sweeps: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SweepDecision {
    pub amount: String,
    pub asset: String,
    #[allow(dead_code)]
    pub rule_type: String,
    pub token_address: Option<Address>,
}

#[derive(Debug, Clone)]
pub struct MockTransaction {
    pub from: Address,
    pub to: Address,
    pub value: String,
    pub asset: String,
    pub nonce: u64,
    pub token_address: Option<Address>,
}

impl MockTransaction {

    pub fn format_log(&self) -> String {
        match &self.token_address {
            Some(token) => {
                format!(
                    "from={}, to={}, value={} {}, token={}, nonce={}",
                    self.from, self.to, self.value, self.asset, token, self.nonce
                )
            }
            None => {
                format!(
                    "from={}, to={}, value={} {}, nonce={}",
                    self.from, self.to, self.value, self.asset, self.nonce
                )
            }
        }
    }
}
