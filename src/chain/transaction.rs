use crate::chain::block::BlockHash;
use crate::crypto::{hash::Hash, signature::Signature, public_key::PublicKey};
use crate::errors::TransactionError;
use crate::types::{Address, Balance, Nonce};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

pub type TransactionHash = Hash;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Transfer,
    StakeDeposit,
    StakeWithdraw,
    AIModelDeploy,
    AIModelInvoke,
    DataValidation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub nonce: Nonce,
    pub from: Address,
    pub to: Address,
    pub value: Balance,
    pub gas_price: u64,
    pub gas_limit: u64,
    pub data: Vec<u8>,
    pub transaction_type: TransactionType,
    pub timestamp: u64,
    pub signature: Option<Signature>,
}

impl Transaction {
    pub fn new(
        nonce: Nonce,
        from: Address,
        to: Address,
        value: Balance,
        gas_price: u64,
        gas_limit: u64,
        data: Vec<u8>,
        transaction_type: TransactionType,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        Self {
            nonce,
            from,
            to,
            value,
            gas_price,
            gas_limit,
            data,
            transaction_type,
            timestamp,
            signature: None,
        }
    }

    pub fn sign(&mut self, private_key: &[u8]) -> Result<(), TransactionError> {
        let message = self.hash()?;
        self.signature = Some(Signature::sign(&message, private_key)?);
        Ok(())
    }

    pub fn verify(&self, public_key: &PublicKey) -> Result<bool, TransactionError> {
        let message = self.hash()?;
        match &self.signature {
            Some(signature) => Ok(signature.verify(&message, public_key)),
            None => Err(TransactionError::MissingSignature),
        }
    }

    pub fn hash(&self) -> Result<TransactionHash, TransactionError> {
        let bytes = bincode::serialize(self)
            .map_err(|_| TransactionError::SerializationError)?;
        Ok(TransactionHash::hash(&bytes))
    }

    pub fn gas_cost(&self) -> u64 {
        self.gas_price * self.gas_limit
    }

    pub fn is_coinbase(&self) -> bool {
        self.from == Address::default() && matches!(self.transaction_type, TransactionType::Transfer)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionReceipt {
    pub transaction_hash: TransactionHash,
    pub block_hash: BlockHash,
    pub block_number: u64,
    pub gas_used: u64,
    pub status: bool,
    pub logs: Vec<Log>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Log {
    pub address: Address,
    pub topics: Vec<Hash>,
    pub data: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::key_pair::KeyPair;

    #[test]
    fn test_transaction_signing_and_verification() {
        let key_pair = KeyPair::generate();
        let mut tx = Transaction::new(
            0,
            Address::random(),
            Address::random(),
            100,
            10,
            21000,
            vec![],
            TransactionType::Transfer,
        );

        tx.sign(key_pair.private_key()).unwrap();
        assert!(tx.verify(key_pair.public_key()).unwrap());

        // Tamper with the transaction
        tx.value += 1;
        assert!(!tx.verify(key_pair.public_key()).unwrap());
    }

    #[test]
    fn test_transaction_hash() {
        let tx = Transaction::new(
            0,
            Address::random(),
            Address::random(),
            100,
            10,
            21000,
            vec![],
            TransactionType::Transfer,
        );

        let hash1 = tx.hash().unwrap();
        let hash2 = tx.hash().unwrap();
        assert_eq!(hash1, hash2);

        let mut tx2 = tx.clone();
        tx2.nonce += 1;
        assert_ne!(tx.hash().unwrap(), tx2.hash().unwrap());
    }

    #[test]
    fn test_coinbase_transaction() {
        let coinbase_tx = Transaction::new(
            0,
            Address::default(),
            Address::random(),
            100,
            0,
            0,
            vec![],
            TransactionType::Transfer,
        );
        assert!(coinbase_tx.is_coinbase());

        let regular_tx = Transaction::new(
            0,
            Address::random(),
            Address::random(),
            100,
            10,
            21000,
            vec![],
            TransactionType::Transfer,
        );
        assert!(!regular_tx.is_coinbase());
    }
}