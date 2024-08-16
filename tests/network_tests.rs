use std::net::SocketAddr;
use tokio::time::Duration;
use omnitensor_core::{
    network::{Network, NetworkConfig, Peer, Message},
    types::{Block, Transaction},
    errors::NetworkError,
};
use mockall::predicate::*;
use mockall::mock;

mock! {
    Peer {}
    impl Peer for Peer {
        fn send(&self, message: Message) -> Result<(), NetworkError>;
        fn address(&self) -> SocketAddr;
    }
}

#[tokio::test]
async fn test_network_initialization() {
    let config = NetworkConfig {
        listen_address: "127.0.0.1:8000".parse().unwrap(),
        max_peers: 10,
        connection_timeout: Duration::from_secs(5),
    };
    
    let network = Network::new(config).await.expect("Failed to initialize network");
    
    assert_eq!(network.peer_count(), 0);
    assert_eq!(network.config().max_peers, 10);
}

#[tokio::test]
async fn test_peer_connection() {
    let mut network = Network::new(NetworkConfig::default()).await.unwrap();
    let mock_peer = MockPeer::new();
    
    network.add_peer(mock_peer).await.expect("Failed to add peer");
    
    assert_eq!(network.peer_count(), 1);
}

#[tokio::test]
async fn test_message_broadcast() {
    let mut network = Network::new(NetworkConfig::default()).await.unwrap();
    
    let mut mock_peer1 = MockPeer::new();
    mock_peer1.expect_send()
        .with(eq(Message::NewBlock(Block::default())))
        .times(1)
        .return_const(Ok(()));
    
    let mut mock_peer2 = MockPeer::new();
    mock_peer2.expect_send()
        .with(eq(Message::NewBlock(Block::default())))
        .times(1)
        .return_const(Ok(()));
    
    network.add_peer(mock_peer1).await.unwrap();
    network.add_peer(mock_peer2).await.unwrap();
    
    network.broadcast(Message::NewBlock(Block::default())).await.unwrap();
}

#[tokio::test]
async fn test_peer_disconnection() {
    let mut network = Network::new(NetworkConfig::default()).await.unwrap();
    let mock_peer = MockPeer::new();
    let peer_addr = "127.0.0.1:8001".parse().unwrap();
    
    network.add_peer(mock_peer).await.unwrap();
    assert_eq!(network.peer_count(), 1);
    
    network.remove_peer(&peer_addr).await.unwrap();
    assert_eq!(network.peer_count(), 0);
}

#[tokio::test]
async fn test_message_handling() {
    let mut network = Network::new(NetworkConfig::default()).await.unwrap();
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);
    
    network.set_message_handler(move |msg| {
        let tx = tx.clone();
        Box::pin(async move {
            tx.send(msg).await.unwrap();
            Ok(())
        })
    });
    
    let mock_peer = MockPeer::new();
    network.add_peer(mock_peer).await.unwrap();
    
    network.handle_incoming_message(
        "127.0.0.1:8001".parse().unwrap(),
        Message::NewTransaction(Transaction::default())
    ).await.unwrap();
    
    let received_msg = rx.recv().await.unwrap();
    assert!(matches!(received_msg, Message::NewTransaction(_)));
}

#[tokio::test]
async fn test_network_stress() {
    let mut network = Network::new(NetworkConfig {
        max_peers: 1000,
        ..Default::default()
    }).await.unwrap();
    
    for _ in 0..1000 {
        let mock_peer = MockPeer::new();
        network.add_peer(mock_peer).await.unwrap();
    }
    
    assert_eq!(network.peer_count(), 1000);
    
    let broadcast_future = network.broadcast(Message::NewBlock(Block::default()));
    tokio::time::timeout(Duration::from_secs(5), broadcast_future)
        .await
        .expect("Broadcast timed out")
        .expect("Broadcast failed");
}