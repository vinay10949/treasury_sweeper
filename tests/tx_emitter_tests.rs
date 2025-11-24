use std::sync::Arc;
use tempfile::TempDir;
use treasury_sweeper::state_manager::StateManager;
use treasury_sweeper::tx_emitter::MockTxEmitter;
use treasury_sweeper::types::SweepDecision;

async fn create_test_emitter() -> (MockTxEmitter, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let state_path = temp_dir.path().join("state.json");
    let state_manager = Arc::new(StateManager::load(state_path).await.unwrap());

    let emitter = MockTxEmitter::new(state_manager, "0xTREASURY".to_string());

    (emitter, temp_dir)
}

#[tokio::test]
async fn test_nonce_increments_correctly() {
    let (emitter, _temp_dir) = create_test_emitter().await;
    let from_addr = "0x1234".to_string();

    let decision = SweepDecision {
        amount: "0.5".to_string(),
        asset: "ETH".to_string(),
        rule_type: "native_balance".to_string(),
        token_address: None,
    };

    let tx1 = emitter.emit_sweep(&from_addr, &decision).await.unwrap();
    assert_eq!(tx1.nonce, 0);

    let tx2 = emitter.emit_sweep(&from_addr, &decision).await.unwrap();
    assert_eq!(tx2.nonce, 1);

    let tx3 = emitter.emit_sweep(&from_addr, &decision).await.unwrap();
    assert_eq!(tx3.nonce, 2);
}

#[tokio::test]
async fn test_different_wallets_independent_nonces() {
    let (emitter, _temp_dir) = create_test_emitter().await;

    let decision = SweepDecision {
        amount: "1.0".to_string(),
        asset: "ETH".to_string(),
        rule_type: "native_balance".to_string(),
        token_address: None,
    };

    // Wallet 1
    let tx1 = emitter.emit_sweep(&"0xWallet1".to_string(), &decision).await.unwrap();
    assert_eq!(tx1.nonce, 0);

    // Wallet 2
    let tx2 = emitter.emit_sweep(&"0xWallet2".to_string(), &decision).await.unwrap();
    assert_eq!(tx2.nonce, 0);

    // Wallet 1 again
    let tx3 = emitter.emit_sweep(&"0xWallet1".to_string(), &decision).await.unwrap();
    assert_eq!(tx3.nonce, 1);

    // Wallet 2 again
    let tx4 = emitter.emit_sweep(&"0xWallet2".to_string(), &decision).await.unwrap();
    assert_eq!(tx4.nonce, 1);
}

#[tokio::test]
async fn test_native_transaction_fields() {
    let (emitter, _temp_dir) = create_test_emitter().await;
    let from_addr = "0xABCD".to_string();

    let decision = SweepDecision {
        amount: "2.5".to_string(),
        asset: "ETH".to_string(),
        rule_type: "native_balance".to_string(),
        token_address: None,
    };

    let tx = emitter.emit_sweep(&from_addr, &decision).await.unwrap();

    assert_eq!(tx.from, "0xABCD");
    assert_eq!(tx.to, "0xTREASURY");
    assert_eq!(tx.value, "2.5");
    assert_eq!(tx.asset, "ETH");
    assert_eq!(tx.nonce, 0);
    assert!(tx.token_address.is_none());
}

#[tokio::test]
async fn test_token_transaction_fields() {
    let (emitter, _temp_dir) = create_test_emitter().await;
    let from_addr = "0x5678".to_string();

    let decision = SweepDecision {
        amount: "150".to_string(),
        asset: "USDC".to_string(),
        rule_type: "token_balance".to_string(),
        token_address: Some("0xUSDC_CONTRACT".to_string()),
    };

    let tx = emitter.emit_sweep(&from_addr, &decision).await.unwrap();

    assert_eq!(tx.from, "0x5678");
    assert_eq!(tx.to, "0xTREASURY");
    assert_eq!(tx.value, "150");
    assert_eq!(tx.asset, "USDC");
    assert_eq!(tx.nonce, 0);
    assert_eq!(tx.token_address, Some("0xUSDC_CONTRACT".to_string()));
}
