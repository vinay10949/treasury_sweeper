//! Dummy Balance Checker
use crate::types::Address;
use anyhow::Result;
use rand::Rng;

pub struct DummyBalanceChecker {
    min: f64,
    max: f64,
}

impl DummyBalanceChecker {
    pub fn new(min: f64, max: f64) -> Self {
        Self { min, max }
    }
    pub async fn check_native_balance(&self, _address: &Address) -> Result<f64> {
        let mut rng = rand::rng();
        let balance = rng.random_range(self.min..self.max);
        Ok(balance)
    }

    pub async fn check_token_balance(
        &self,
        _address: &Address,
        _token_address: &Address,
    ) -> Result<u64> {
        let mut rng = rand::rng();
        let balance = rng.random_range(100_u64..200_u64);
        Ok(balance)
    }
}
