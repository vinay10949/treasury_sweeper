use treasury_sweeper::balance_checker::DummyBalanceChecker;
use treasury_sweeper::rules_engine::RulesEngine;
use treasury_sweeper::types::{HotWalletConfig, SweepRule};

fn create_test_wallet(rules: Vec<SweepRule>) -> HotWalletConfig {
    HotWalletConfig {
        address: "0xTestWallet".to_string(),
        label: "Test Wallet".to_string(),
        rules,
    }
}

#[tokio::test]
async fn test_native_balance_rule_triggers() {
    let checker = DummyBalanceChecker::new(0.5, 1.0);
    let engine = RulesEngine::new(checker);

    let wallet = create_test_wallet(vec![SweepRule::NativeBalance {
        threshold: "0.1".to_string(),
        asset: "ETH".to_string(),
    }]);

    let decisions = engine.evaluate(&wallet).await.unwrap();
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].asset, "ETH");
    assert_eq!(decisions[0].rule_type, "native_balance");
    assert!(decisions[0].token_address.is_none());
}

#[tokio::test]
async fn test_native_balance_rule_no_trigger() {
    let checker = DummyBalanceChecker::new(0.01, 0.05);
    let engine = RulesEngine::new(checker);

    let wallet = create_test_wallet(vec![SweepRule::NativeBalance {
        threshold: "0.1".to_string(),
        asset: "ETH".to_string(),
    }]);

    let decisions = engine.evaluate(&wallet).await.unwrap();
    assert_eq!(decisions.len(), 0);
}

#[tokio::test]
async fn test_token_balance_rule_triggers() {
    let checker = DummyBalanceChecker::new(0.0, 1.0);
    let engine = RulesEngine::new(checker);

    let wallet = create_test_wallet(vec![SweepRule::TokenBalance {
        threshold: "50".to_string(),
        token_address: "0xUSDC".to_string(),
        asset: "USDC".to_string(),
    }]);

    let decisions = engine.evaluate(&wallet).await.unwrap();
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].asset, "USDC");
    assert_eq!(decisions[0].rule_type, "token_balance");
    assert_eq!(
        decisions[0].token_address,
        Some("0xUSDC".to_string())
    );
}

#[tokio::test]
async fn test_multiple_rules_all_trigger() {
    let checker = DummyBalanceChecker::new(0.5, 1.0);
    let engine = RulesEngine::new(checker);

    let wallet = create_test_wallet(vec![
        SweepRule::NativeBalance {
            threshold: "0.1".to_string(),
            asset: "ETH".to_string(),
        },
        SweepRule::TokenBalance {
            threshold: "50".to_string(),
            token_address: "0xUSDC".to_string(),
            asset: "USDC".to_string(),
        },
        SweepRule::TokenBalance {
            threshold: "75".to_string(),
            token_address: "0xDAI".to_string(),
            asset: "DAI".to_string(),
        },
    ]);

    let decisions = engine.evaluate(&wallet).await.unwrap();
    assert_eq!(decisions.len(), 3);
    assert_eq!(decisions[0].rule_type, "native_balance");
    assert_eq!(decisions[1].rule_type, "token_balance");
    assert_eq!(decisions[2].rule_type, "token_balance");
}

#[tokio::test]
async fn test_multiple_rules_partial_trigger() {
    let checker = DummyBalanceChecker::new(0.01, 0.05);
    let engine = RulesEngine::new(checker);

    let wallet = create_test_wallet(vec![
        SweepRule::NativeBalance {
            threshold: "0.1".to_string(),
            asset: "ETH".to_string(),
        },
        SweepRule::TokenBalance {
            threshold: "50".to_string(),
            token_address: "0xUSDC".to_string(),
            asset: "USDC".to_string(),
        },
    ]);

    let decisions = engine.evaluate(&wallet).await.unwrap();
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].rule_type, "token_balance");
}
