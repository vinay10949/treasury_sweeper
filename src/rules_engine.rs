//! Rules Engine
//!
//! Evaluates sweep rules against wallet balances to determine if a sweep should be triggered.
use crate::balance_checker::DummyBalanceChecker;
use crate::types::{HotWalletConfig, SweepDecision, SweepRule};
use anyhow::Result;
use tracing::info;


pub struct RulesEngine {
    balance_checker: DummyBalanceChecker,
}

impl RulesEngine {
    pub fn new(balance_checker: DummyBalanceChecker) -> Self {
        Self { balance_checker }
    }

    /// Evaluate all rules for a wallet and return all sweep decisions that trigger
    pub async fn evaluate(&self, wallet_config: &HotWalletConfig) -> Result<Vec<SweepDecision>> {
        info!("Evaluating rules for wallet {}", &wallet_config.address);
        
        let mut decisions = Vec::new();
        
        for rule in &wallet_config.rules {
            match rule {
                SweepRule::NativeBalance { threshold, asset } => {
                    let balance = self
                        .balance_checker
                        .check_native_balance(&wallet_config.address)
                        .await?;

                    let threshold_value: f64 = threshold.parse().unwrap_or(0.0);

                    info!(
                        "Balance check: {}={:.6} (threshold={})",
                        asset, balance, threshold
                    );

                    if balance > threshold_value {
                        info!("Rule triggered: native_balance");
                        decisions.push(SweepDecision {
                            amount: balance.to_string(),
                            asset: asset.clone(),
                            rule_type: "native_balance".to_string(),
                            token_address: None,
                        });
                    }
                }

                SweepRule::TokenBalance {
                    threshold,
                    token_address,
                    asset,
                } => {
                    let balance = self
                        .balance_checker
                        .check_token_balance(&wallet_config.address, token_address)
                        .await?;

                    let threshold_value: u64 = threshold.parse().unwrap_or(0);

                    info!(
                        "Balance check: {}={} (threshold={}, token={})",
                        asset, balance, threshold, token_address
                    );

                    if balance > threshold_value {
                        info!("Rule triggered: token_balance");
                        decisions.push(SweepDecision {
                            amount: balance.to_string(),
                            asset: asset.clone(),
                            rule_type: "token_balance".to_string(),
                            token_address: Some(token_address.clone()),
                        });
                    }
                }
            }
        }

        if decisions.is_empty() {
            info!("No rules triggered, skipping");
        } else {
            info!("No of rules triggered :{}", decisions.len());
        }
        
        Ok(decisions)
    }
}

