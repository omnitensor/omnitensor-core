# OmniTensor Core API Documentation

## Blockchain Module (chain)

### Block
- **Structure**: Represents a single block in the blockchain.
- **Fields**:
  - `index: u64` - The block's index in the chain.
  - `prev_hash: String` - Hash of the previous block.
  - `timestamp: u64` - Time when the block was created.
  - `transactions: Vec<String>` - List of transactions in the block.
  - `nonce: u64` - Proof-of-Work nonce.
  - `hash: String` - Hash of the block.
- **Methods**:
  - `new(index, prev_hash, transactions, nonce, timestamp) -> Block`
  - `calculate_hash() -> String`

### Transaction
- **Structure**: Represents a single transaction in the blockchain.
- **Fields**:
  - `from: String` - Sender's address.
  - `to: String` - Receiver's address.
  - `amount: u64` - Amount being transferred.
  - `data: Option<String>` - Additional data for the transaction.
- **Methods**:
  - `new(from, to, amount, data) -> Transaction`

---

## Consensus Module (consensus)

### Validator
- **Purpose**: Ensures the validity of blocks in the blockchain.
- **Fields**:
  - `staking_amount: u64` - Amount staked by the validator.
- **Methods**:
  - `new(staking_amount) -> Validator`
  - `validate_block(block_hash: &str) -> bool`

### StakeManager
- **Purpose**: Manages staking-related operations.
- **Fields**:
  - `total_stake: u64` - Total amount of stakes in the system.
- **Methods**:
  - `new() -> StakeManager`
  - `add_stake(amount: u64)`
  - `get_total_stake() -> u64`

---

## Network Module (network)

### P2PNetwork
- **Purpose**: Manages peer-to-peer connections.
- **Fields**:
  - `peers: Vec<String>` - List of connected peers.
- **Methods**:
  - `new() -> P2PNetwork`
  - `connect(addr: &str)`

### SyncManager
- **Purpose**: Handles synchronization of blockchain data.
- **Fields**:
  - `synced_blocks: u64` - Number of synced blocks.
- **Methods**:
  - `new() -> SyncManager`
  - `sync_block()`
