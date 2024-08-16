use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, warn, error};
use futures::stream::StreamExt;

use crate::types::{Block, BlockHeader, Transaction};
use crate::network::peer::{Peer, PeerManager};
use crate::chain::Chain;
use crate::consensus::ConsensusEngine;

pub struct Synchronizer {
    chain: Arc<RwLock<Chain>>,
    peer_manager: Arc<PeerManager>,
    consensus_engine: Arc<ConsensusEngine>,
}

impl Synchronizer {
    pub fn new(chain: Arc<RwLock<Chain>>, peer_manager: Arc<PeerManager>, consensus_engine: Arc<ConsensusEngine>) -> Self {
        Self {
            chain,
            peer_manager,
            consensus_engine,
        }
    }

    pub async fn start(&self) {
        info!("Starting synchronizer");
        loop {
            self.sync_with_network().await;
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        }
    }

    async fn sync_with_network(&self) {
        let peers = self.peer_manager.get_active_peers().await;
        if peers.is_empty() {
            warn!("No active peers available for synchronization");
            return;
        }

        let local_height = self.chain.read().await.get_height();
        let mut highest_peer = None;
        let mut highest_height = local_height;

        for peer in peers {
            match peer.get_height().await {
                Ok(peer_height) if peer_height > highest_height => {
                    highest_height = peer_height;
                    highest_peer = Some(peer);
                }
                Ok(_) => {}
                Err(e) => warn!("Failed to get height from peer: {}", e),
            }
        }

        if let Some(peer) = highest_peer {
            self.sync_with_peer(peer, local_height, highest_height).await;
        }
    }

    async fn sync_with_peer(&self, peer: Arc<Peer>, start_height: u64, end_height: u64) {
        info!("Syncing with peer from height {} to {}", start_height, end_height);

        let mut current_height = start_height;
        while current_height < end_height {
            match self.fetch_block_range(peer.clone(), current_height, current_height + 100).await {
                Ok(blocks) => {
                    for block in blocks {
                        if let Err(e) = self.process_block(block).await {
                            error!("Failed to process block: {}", e);
                            return;
                        }
                        current_height += 1;
                    }
                }
                Err(e) => {
                    error!("Failed to fetch block range: {}", e);
                    return;
                }
            }
        }

        info!("Sync completed successfully");
    }

    async fn fetch_block_range(&self, peer: Arc<Peer>, start: u64, end: u64) -> Result<Vec<Block>, Box<dyn std::error::Error>> {
        let headers = peer.get_block_headers(start, end).await?;
        let mut blocks = Vec::new();

        for header in headers {
            let transactions = peer.get_block_transactions(header.hash).await?;
            blocks.push(Block::new(header, transactions));
        }

        Ok(blocks)
    }

    async fn process_block(&self, block: Block) -> Result<(), Box<dyn std::error::Error>> {
        let mut chain = self.chain.write().await;

        // Verify block
        self.consensus_engine.verify_block(&block, &chain).await?;

        // Apply transactions
        for tx in &block.transactions {
            chain.apply_transaction(tx).await?;
        }

        // Add block to chain
        chain.add_block(block).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{create_test_chain, create_test_peer_manager, create_test_consensus_engine};

    #[tokio::test]
    async fn test_sync_with_peer() {
        let chain = Arc::new(RwLock::new(create_test_chain()));
        let peer_manager = Arc::new(create_test_peer_manager());
        let consensus_engine = Arc::new(create_test_consensus_engine());

        let synchronizer = Synchronizer::new(chain.clone(), peer_manager.clone(), consensus_engine);

        let test_peer = peer_manager.get_active_peers().await[0].clone();
        synchronizer.sync_with_peer(test_peer, 0, 100).await;

        assert_eq!(chain.read().await.get_height(), 100);
    }
}