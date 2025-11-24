use std::sync::Arc;
use tempfile::TempDir;
use treasury_sweeper::balance_checker::DummyBalanceChecker;
use treasury_sweeper::monitor::WalletMonitor;
use treasury_sweeper::rules_engine::RulesEngine;
use treasury_sweeper::state_manager::StateManager;
use treasury_sweeper::tx_emitter::MockTxEmitter;
use treasury_sweeper::types::{Config, HotWalletConfig, SweepRule};

async fn create_test_monitor() -> (WalletMonitor, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let state_path = temp_dir.path().join("state.json");
    let state_manager = Arc::new(StateManager::load(state_path).await.unwrap());

    let balance_checker = DummyBalanceChecker::new(0.5, 1.0);
    let rules_engine = Arc::new(RulesEngine::new(balance_checker));

    let tx_emitter = Arc::new(MockTxEmitter::new(
        state_manager,
        "0xTREASURY".to_string(),
    ));

    let monitor = WalletMonitor::new(rules_engine, tx_emitter);

    (monitor, temp_dir)
}

#[tokio::test]
async fn test_wallet_native_balance_sweep() {
    let (monitor, _temp_dir) = create_test_monitor().await;

    let config = Config {
        treasury_address: "0xTREASURY".to_string(),
        hot_wallets: vec![HotWalletConfig {
            address: "0x1234".to_string(),
            label: "Test Wallet".to_string(),
            rules: vec![SweepRule::NativeBalance {
                threshold: "0.1".to_string(),
                asset: "ETH".to_string(),
            }],
        }],
        sweep_interval_seconds: 60,
    };

    let sweep_count = monitor.check_all_wallets(&config).await.unwrap();
    assert_eq!(sweep_count, 1);
}


#[tokio::test]
async fn test_multiple_rules_per_wallet() {
    let (monitor, _temp_dir) = create_test_monitor().await;

    let config = Config {
        treasury_address: "0xTREASURY".to_string(),
        hot_wallets: vec![HotWalletConfig {
            address: "0x1234".to_string(),
            label: "Multi-Rule Wallet".to_string(),
            rules: vec![
                SweepRule::NativeBalance {
                    threshold: "0.1".to_string(),
                    asset: "ETH".to_string(),
                },
                SweepRule::TokenBalance {
                    threshold: "50".to_string(),
                    token_address: "0xUSDC".to_string(),
                    asset: "USDC".to_string(),
                },
            ],
        }],
        sweep_interval_seconds: 60,
    };

    let sweep_count = monitor.check_all_wallets(&config).await.unwrap();
    assert_eq!(sweep_count, 2);
}

#[tokio::test]
async fn test_no_sweep_when_threshold_not_met() {
    let (monitor, _temp_dir) = create_test_monitor().await;

    let config = Config {
        treasury_address: "0xTREASURY".to_string(),
        hot_wallets: vec![HotWalletConfig {
            address: "0x1234".to_string(),
            label: "High Threshold Wallet".to_string(),
            rules: vec![SweepRule::NativeBalance {
                threshold: "10.0".to_string(),
                asset: "ETH".to_string(),
            }],
        }],
        sweep_interval_seconds: 60,
    };

    let sweep_count = monitor.check_all_wallets(&config).await.unwrap();
    assert_eq!(sweep_count, 0);
}
