use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::convert::TryInto;

use crate::chain::transaction::Transaction;
use crate::consensus::proof::Proof;
use crate::errors::BlockError;

const MAX_TRANSACTIONS: usize = 1000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub version: u32,
    pub prev_block_hash: [u8; 32],
    pub merkle_root: [u8; 32],
    pub timestamp: i64,
    pub difficulty: u32,
    pub nonce: u64,
}

impl Block {
    pub fn new(prev_block_hash: [u8; 32], transactions: Vec<Transaction>, difficulty: u32) -> Result<Self, BlockError> {
        if transactions.len() > MAX_TRANSACTIONS {
            return Err(BlockError::TooManyTransactions);
        }

        let merkle_root = Self::calculate_merkle_root(&transactions);
        
        Ok(Block {
            header: BlockHeader {
                version: 1,
                prev_block_hash,
                merkle_root,
                timestamp: Utc::now().timestamp(),
                difficulty,
                nonce: 0,
            },
            transactions,
        })
    }

    pub fn mine(&mut self) -> Proof {
        let mut proof = Proof::new(&self.header);
        while !proof.is_valid(self.header.difficulty) {
            self.header.nonce += 1;
            proof = Proof::new(&self.header);
        }
        proof
    }

    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha3_256::new();
        hasher.update(bincode::serialize(&self.header).unwrap());
        hasher.finalize().try_into().unwrap()
    }

    fn calculate_merkle_root(transactions: &[Transaction]) -> [u8; 32] {
        if transactions.is_empty() {
            return [0; 32];
        }

        let mut hashes: Vec<[u8; 32]> = transactions
            .iter()
            .map(|tx| tx.hash())
            .collect();

        while hashes.len() > 1 {
            let mut next_level = Vec::new();
            for chunk in hashes.chunks(2) {
                let mut hasher = Sha3_256::new();
                hasher.update(&chunk[0]);
                if chunk.len() > 1 {
                    hasher.update(&chunk[1]);
                } else {
                    hasher.update(&chunk[0]);  // If odd number, duplicate last hash
                }
                next_level.push(hasher.finalize().try_into().unwrap());
            }
            hashes = next_level;
        }

        hashes[0]
    }

    pub fn validate(&self) -> Result<(), BlockError> {
        if self.transactions.len() > MAX_TRANSACTIONS {
            return Err(BlockError::TooManyTransactions);
        }

        let calculated_merkle_root = Self::calculate_merkle_root(&self.transactions);
        if calculated_merkle_root != self.header.merkle_root {
            return Err(BlockError::InvalidMerkleRoot);
        }

        let proof = Proof::new(&self.header);
        if !proof.is_valid(self.header.difficulty) {
            return Err(BlockError::InvalidProof);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::transaction::Transaction;

    #[test]
    fn test_new_block() {
        let prev_block_hash = [0; 32];
        let transactions = vec![Transaction::new(vec![0, 1, 2], vec![3, 4, 5])];
        let difficulty = 1;

        let block = Block::new(prev_block_hash, transactions, difficulty).unwrap();

        assert_eq!(block.header.version, 1);
        assert_eq!(block.header.prev_block_hash, prev_block_hash);
        assert_eq!(block.header.difficulty, difficulty);
        assert_eq!(block.transactions.len(), 1);
    }

    #[test]
    fn test_mine_block() {
        let mut block = Block::new([0; 32], vec![], 1).unwrap();
        let proof = block.mine();
        assert!(proof.is_valid(block.header.difficulty));
    }

    #[test]
    fn test_validate_block() {
        let mut block = Block::new([0; 32], vec![], 1).unwrap();
        block.mine();
        assert!(block.validate().is_ok());

        // Tamper with the block
        block.header.merkle_root = [1; 32];
        assert!(block.validate().is_err());
    }
}