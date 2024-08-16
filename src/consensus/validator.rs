use std::sync::{Arc, Mutex};
use tokio::time::{Duration, interval};
use log::{info, error, warn};

use crate::types::{Block, Transaction, Hash};
use crate::network::P2PNetwork;
use crate::storage::BlockchainDB;
use crate::crypto::{sign, verify_signature};

pub struct Validator {
    node_id: String,
    stake: u64,
    private_key: Vec<u8>,
    network: Arc<P2PNetwork>,
    blockchain: Arc<Mutex<BlockchainDB>>,
}

impl Validator {
    pub fn new(node_id: String, stake: u64, private_key: Vec<u8>, network: Arc<P2PNetwork>, blockchain: Arc<Mutex<BlockchainDB>>) -> Self {
        Validator {
            node_id,
            stake,
            private_key,
            network,
            blockchain,
        }
    }

    pub async fn start(&self) {
        let mut interval = interval(Duration::from_secs(10)); // Validate every 10 seconds

        loop {
            interval.tick().await;
            self.validate_and_propose_block().await;
        }
    }

    async fn validate_and_propose_block(&self) {
        let pending_transactions = self.network.get_pending_transactions().await;
        
        if pending_transactions.is_empty() {
            info!("No pending transactions to validate.");
            return;
        }

        let valid_transactions = self.validate_transactions(&pending_transactions);
        
        if valid_transactions.is_empty() {
            warn!("No valid transactions found in the pending pool.");
            return;
        }

        let new_block = self.create_block(valid_transactions);
        
        if let Err(e) = self.propose_block(new_block).await {
            error!("Failed to propose new block: {:?}", e);
        }
    }

    fn validate_transactions(&self, transactions: &[Transaction]) -> Vec<Transaction> {
        transactions.iter()
            .filter(|tx| self.is_transaction_valid(tx))
            .cloned()
            .collect()
    }

    fn is_transaction_valid(&self, transaction: &Transaction) -> bool {
        // Implement transaction validation logic here
        // Check signature, balance, nonce, etc.
        verify_signature(&transaction.from, &transaction.data, &transaction.signature)
    }

    fn create_block(&self, transactions: Vec<Transaction>) -> Block {
        let prev_block = self.blockchain.lock().unwrap().get_latest_block();
        
        Block {
            header: BlockHeader {
                prev_hash: prev_block.hash,
                timestamp: chrono::Utc::now().timestamp(),
                merkle_root: self.calculate_merkle_root(&transactions),
                validator: self.node_id.clone(),
            },
            transactions,
            signature: Vec::new(), // To be filled after signing
        }
    }

    async fn propose_block(&self, mut block: Block) -> Result<(), Box<dyn std::error::Error>> {
        let block_hash = block.calculate_hash();
        let signature = sign(&self.private_key, &block_hash);
        block.signature = signature;

        self.network.broadcast_block(block.clone()).await?;

        if let Err(e) = self.blockchain.lock().unwrap().add_block(block) {
            error!("Failed to add proposed block to local chain: {:?}", e);
            return Err(Box::new(e));
        }

        info!("Successfully proposed and added new block: {:?}", block_hash);
        Ok(())
    }

    fn calculate_merkle_root(&self, transactions: &[Transaction]) -> Hash {
        // Implement Merkle root calculation
        // This is a placeholder implementation
        let mut hasher = sha2::Sha256::new();
        for tx in transactions {
            hasher.update(tx.hash.as_bytes());
        }
        Hash::from(hasher.finalize().as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{generate_test_transactions, setup_test_network, setup_test_blockchain};

    #[tokio::test]
    async fn test_validate_and_propose_block() {
        let network = setup_test_network();
        let blockchain = setup_test_blockchain();
        let validator = Validator::new(
            "test_validator".to_string(),
            1000,
            vec![0; 32], // dummy private key
            Arc::new(network),
            Arc::new(Mutex::new(blockchain)),
        );

        let test_transactions = generate_test_transactions(10);
        network.add_pending_transactions(test_transactions).await;

        validator.validate_and_propose_block().await;

        let latest_block = validator.blockchain.lock().unwrap().get_latest_block();
        assert!(!latest_block.transactions.is_empty(), "Block should contain transactions");
        assert_eq!(latest_block.header.validator, "test_validator", "Block should be proposed by test validator");
    }

    // Add more unit tests here
}