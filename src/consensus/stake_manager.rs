use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::types::{Address, Balance, BlockHeight};
use crate::crypto::hash::Hash;
use crate::storage::Storage;

#[derive(Debug, Serialize, Deserialize)]
pub struct Stake {
    amount: Balance,
    staked_at: DateTime<Utc>,
    last_reward_height: BlockHeight,
}

#[derive(Debug, Error)]
pub enum StakeManagerError {
    #[error("Insufficient balance for staking")]
    InsufficientBalance,
    #[error("Stake not found for address")]
    StakeNotFound,
    #[error("Storage error: {0}")]
    StorageError(#[from] crate::storage::StorageError),
}

pub struct StakeManager<S: Storage> {
    storage: S,
    min_stake: Balance,
    reward_rate: f64,
}

impl<S: Storage> StakeManager<S> {
    pub fn new(storage: S, min_stake: Balance, reward_rate: f64) -> Self {
        Self {
            storage,
            min_stake,
            reward_rate,
        }
    }

    pub fn stake(&mut self, address: Address, amount: Balance) -> Result<(), StakeManagerError> {
        if amount < self.min_stake {
            return Err(StakeManagerError::InsufficientBalance);
        }

        let mut stakes = self.get_stakes()?;
        let stake = stakes.entry(address).or_insert(Stake {
            amount: Balance::zero(),
            staked_at: Utc::now(),
            last_reward_height: BlockHeight::zero(),
        });

        stake.amount += amount;
        self.storage.set(b"stakes", &stakes)?;

        Ok(())
    }

    pub fn unstake(&mut self, address: Address, amount: Balance) -> Result<Balance, StakeManagerError> {
        let mut stakes = self.get_stakes()?;
        let stake = stakes.get_mut(&address).ok_or(StakeManagerError::StakeNotFound)?;

        if stake.amount < amount {
            return Err(StakeManagerError::InsufficientBalance);
        }

        stake.amount -= amount;
        if stake.amount.is_zero() {
            stakes.remove(&address);
        }

        self.storage.set(b"stakes", &stakes)?;

        Ok(amount)
    }

    pub fn calculate_rewards(&self, address: Address, current_height: BlockHeight) -> Result<Balance, StakeManagerError> {
        let stakes = self.get_stakes()?;
        let stake = stakes.get(&address).ok_or(StakeManagerError::StakeNotFound)?;

        let blocks_since_last_reward = current_height - stake.last_reward_height;
        let reward = (stake.amount.as_f64() * self.reward_rate * blocks_since_last_reward.as_f64()).round() as u64;

        Ok(Balance::from(reward))
    }

    pub fn distribute_rewards(&mut self, current_height: BlockHeight) -> Result<(), StakeManagerError> {
        let mut stakes = self.get_stakes()?;

        for (address, stake) in stakes.iter_mut() {
            let reward = self.calculate_rewards(*address, current_height)?;
            stake.amount += reward;
            stake.last_reward_height = current_height;
        }

        self.storage.set(b"stakes", &stakes)?;

        Ok(())
    }

    fn get_stakes(&self) -> Result<HashMap<Address, Stake>, StakeManagerError> {
        self.storage
            .get(b"stakes")
            .map(|v| v.unwrap_or_default())
            .map_err(StakeManagerError::from)
    }

    pub fn get_total_staked(&self) -> Result<Balance, StakeManagerError> {
        let stakes = self.get_stakes()?;
        Ok(stakes.values().map(|stake| stake.amount).sum())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MemoryStorage;

    #[test]
    fn test_stake_and_unstake() {
        let storage = MemoryStorage::new();
        let mut stake_manager = StakeManager::new(storage, Balance::from(100), 0.001);

        let address = Address::random();
        
        // Test staking
        stake_manager.stake(address, Balance::from(500)).unwrap();
        assert_eq!(stake_manager.get_total_staked().unwrap(), Balance::from(500));

        // Test unstaking
        let unstaked = stake_manager.unstake(address, Balance::from(200)).unwrap();
        assert_eq!(unstaked, Balance::from(200));
        assert_eq!(stake_manager.get_total_staked().unwrap(), Balance::from(300));
    }

    #[test]
    fn test_rewards_calculation() {
        let storage = MemoryStorage::new();
        let mut stake_manager = StakeManager::new(storage, Balance::from(100), 0.001);

        let address = Address::random();
        stake_manager.stake(address, Balance::from(1000)).unwrap();

        let reward = stake_manager.calculate_rewards(address, BlockHeight::from(100)).unwrap();
        assert_eq!(reward, Balance::from(100)); // 1000 * 0.001 * 100 = 100

        stake_manager.distribute_rewards(BlockHeight::from(100)).unwrap();
        assert_eq!(stake_manager.get_total_staked().unwrap(), Balance::from(1100));
    }
}