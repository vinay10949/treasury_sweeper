use std::sync::Arc;
use tempfile::TempDir;
use treasury_sweeper::state_manager::StateManager;

#[tokio::test]
async fn test_state_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let state_path = temp_dir.path().join("state.json");

    // Create and use state manager
    {
        let state_manager = StateManager::load(state_path.clone()).await.unwrap();

        // Reserve some nonces
        let nonce1 = state_manager.reserve_nonce(&"0xWallet1".to_string()).await.unwrap();
        assert_eq!(nonce1, 0);

        let nonce2 = state_manager.reserve_nonce(&"0xWallet1".to_string()).await.unwrap();
        assert_eq!(nonce2, 1);

        let nonce3 = state_manager.reserve_nonce(&"0xWallet2".to_string()).await.unwrap();
        assert_eq!(nonce3, 0);
    }

    // Load state again and verify persistence
    {
        let state_manager = StateManager::load(state_path.clone()).await.unwrap();

        let nonce4 = state_manager.reserve_nonce(&"0xWallet1".to_string()).await.unwrap();
        assert_eq!(nonce4, 2);

        let nonce5 = state_manager.reserve_nonce(&"0xWallet2".to_string()).await.unwrap();
        assert_eq!(nonce5, 1);
    }
}

#[tokio::test]
async fn test_concurrent_nonce_reservation() {
    let temp_dir = TempDir::new().unwrap();
    let state_path = temp_dir.path().join("state.json");
    let state_manager = Arc::new(StateManager::load(state_path).await.unwrap());

    let wallet_address = "0xConcurrent".to_string();

    let mut handles = vec![];
    for _ in 0..10 {
        let sm = Arc::clone(&state_manager);
        let addr = wallet_address.clone();
        let handle = tokio::spawn(async move { sm.reserve_nonce(&addr).await.unwrap() });
        handles.push(handle);
    }

    let mut nonces = vec![];
    for handle in handles {
        nonces.push(handle.await.unwrap());
    }

    assert_eq!(nonces, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
}

#[tokio::test]
async fn test_state_snapshot() {
    let temp_dir = TempDir::new().unwrap();
    let state_path = temp_dir.path().join("state.json");
    let state_manager = StateManager::load(state_path).await.unwrap();

    // Reserve some nonces
    state_manager.reserve_nonce(&"0xWallet1".to_string()).await.unwrap();
    state_manager.reserve_nonce(&"0xWallet1".to_string()).await.unwrap();
    state_manager.reserve_nonce(&"0xWallet2".to_string()).await.unwrap();

    // Get snapshot
    let snapshot = state_manager.fetch_snapshot().await;

    // Verify snapshot contains correct data
    assert_eq!(snapshot.wallets.len(), 2);
    assert_eq!(snapshot.wallets.get("0xWallet1").unwrap().next_nonce, 2);
    assert_eq!(snapshot.wallets.get("0xWallet1").unwrap().total_sweeps, 2);
    assert_eq!(snapshot.wallets.get("0xWallet2").unwrap().next_nonce, 1);
    assert_eq!(snapshot.wallets.get("0xWallet2").unwrap().total_sweeps, 1);
}
