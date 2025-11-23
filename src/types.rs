//! Core data types for the Treasury Sweeper Service

use rand::Rng;
use serde::{Deserialize, Serialize};

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

pub fn truncate_address(address: &str) -> String {
    if address.len() <= 10 {
        return address.to_string();
    }
    format!("{}...{}", &address[..6], &address[address.len() - 4..])
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
