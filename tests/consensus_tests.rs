use omnitensor_core::consensus::{Validator, StakeManager, ConsensusEngine};
use omnitensor_core::types::{Block, Transaction, Address};
use omnitensor_core::config::ConsensusConfig;
use tokio;
use std::sync::Arc;
use parking_lot::RwLock;

#[cfg(test)]
mod consensus_tests {
    use super::*;
    use anyhow::Result;

    async fn setup_consensus_engine() -> Arc<ConsensusEngine> {
        let config = ConsensusConfig::default();
        let stake_manager = Arc::new(RwLock::new(StakeManager::new(config.clone())));
        Arc::new(ConsensusEngine::new(config, stake_manager))
    }

    #[tokio::test]
    async fn test_validator_registration() -> Result<()> {
        let engine = setup_consensus_engine().await;
        let validator_address = Address::random();
        let stake_amount = 1000;

        engine.register_validator(validator_address, stake_amount).await?;
        
        let validators = engine.get_active_validators().await;
        assert!(validators.contains(&validator_address));
        Ok(())
    }

    #[tokio::test]
    async fn test_block_proposal() -> Result<()> {
        let engine = setup_consensus_engine().await;
        let proposer = Address::random();
        engine.register_validator(proposer, 1000).await?;

        let transactions = vec![
            Transaction::new(Address::random(), Address::random(), 100),
            Transaction::new(Address::random(), Address::random(), 200),
        ];

        let block = Block::new(1, proposer, transactions);
        
        engine.propose_block(block.clone()).await?;
        assert_eq!(engine.get_latest_block().await?, block);
        Ok(())
    }

    #[tokio::test]
    async fn test_consensus_round() -> Result<()> {
        let engine = setup_consensus_engine().await;
        
        // Register multiple validators
        for _ in 0..4 {
            let validator = Address::random();
            engine.register_validator(validator, 1000).await?;
        }

        let proposer = engine.select_proposer().await?;
        let block = Block::new(1, proposer, vec![]);

        engine.propose_block(block.clone()).await?;

        // Simulate voting
        for validator in engine.get_active_validators().await {
            engine.vote(validator, block.hash()).await?;
        }

        // Check if consensus is reached
        assert!(engine.is_consensus_reached(&block).await);
        assert_eq!(engine.get_latest_block().await?, block);
        Ok(())
    }

    #[tokio::test]
    async fn test_fork_choice_rule() -> Result<()> {
        let engine = setup_consensus_engine().await;
        
        // Create two competing chains
        let fork_a = vec![
            Block::new(1, Address::random(), vec![]),
            Block::new(2, Address::random(), vec![]),
        ];

        let fork_b = vec![
            Block::new(1, Address::random(), vec![]),
            Block::new(2, Address::random(), vec![]),
            Block::new(3, Address::random(), vec![]),
        ];

        // Add both forks to the engine
        for block in fork_a.iter().chain(fork_b.iter()) {
            engine.add_block(block.clone()).await?;
        }

        // Check that the longest chain is selected
        assert_eq!(engine.get_canonical_chain().await?, fork_b);
        Ok(())
    }

    #[tokio::test]
    async fn test_slashing() -> Result<()> {
        let engine = setup_consensus_engine().await;
        let malicious_validator = Address::random();
        
        engine.register_validator(malicious_validator, 1000).await?;
        
        // Simulate double voting
        let block1 = Block::new(1, Address::random(), vec![]);
        let block2 = Block::new(1, Address::random(), vec![]);
        
        engine.vote(malicious_validator, block1.hash()).await?;
        let result = engine.vote(malicious_validator, block2.hash()).await;
        
        assert!(result.is_err());
        assert!(!engine.get_active_validators().await.contains(&malicious_validator));
        Ok(())
    }
}