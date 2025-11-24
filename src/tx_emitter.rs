//! Mock Transaction Emitter
//!
//! Simulates transaction building and submission.
//! In a real implementation, this would sign and broadcast transactions to the blockchain.

use crate::state_manager::StateManager;
use crate::types::{Address, MockTransaction, SweepDecision};
use anyhow::Result;
use std::sync::Arc;
use tracing::{info};

/// Mock transaction emitter
pub struct MockTxEmitter {
    state_manager: Arc<StateManager>,
    treasury_address: Address,
}

impl MockTxEmitter {
    pub fn new(state_manager: Arc<StateManager>, treasury_address: Address) -> Self {
        Self {
            state_manager,
            treasury_address,
        }
    }


    /// Emit a sweep transaction
    pub async fn emit_sweep(
        &self,
        from_address: &Address,
        decision: &SweepDecision,
    ) -> Result<MockTransaction> {
        let nonce = self.state_manager.reserve_nonce(from_address).await?;

        // Step 2: Build mock transaction
        let tx = MockTransaction {
            from: from_address.clone(),
            to: self.treasury_address.clone(),
            value: decision.amount.clone(),
            asset: decision.asset.clone(),
            nonce,
            token_address: decision.token_address.clone(),
        };

        info!("GENERATING TX: {}", tx.format_log());

            info!(
                "SWEEP SUBMITTED: {} {} from {} to {}",
                decision.amount,
                decision.asset,
                from_address,
                self.treasury_address
            );
        Ok(tx)
    }
}
