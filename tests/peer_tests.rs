//! Integration tests for peer management.

mod common;

use common::test_addr;
use parlance::core::peer::{Peer, PeerRegistry};
use std::time::Duration;

const TEST_TIMEOUT: Duration = Duration::from_secs(15);

#[tokio::test]
async fn test_peer_registry_upsert() {
    let registry = PeerRegistry::new();
    let addr = test_addr(8080);
    let peer = Peer::new("Alice".to_string(), addr);

    registry.upsert(peer.clone()).await;

    let peers = registry.get_all().await;
    assert_eq!(peers.len(), 1);
    assert_eq!(peers[0].nickname, "Alice");
}

#[tokio::test]
async fn test_peer_registry_multiple_peers() {
    let registry = PeerRegistry::new();

    let addr1 = test_addr(8080);
    let peer1 = Peer::new("Alice".to_string(), addr1);

    let addr2 = test_addr(8081);
    let peer2 = Peer::new("Bob".to_string(), addr2);

    registry.upsert(peer1).await;
    registry.upsert(peer2).await;

    let peers = registry.get_all().await;
    assert_eq!(peers.len(), 2);
}

#[tokio::test]
async fn test_peer_registry_update_existing() {
    let registry = PeerRegistry::new();
    let addr = test_addr(8080);

    let peer1 = Peer::new("Alice".to_string(), addr);
    registry.upsert(peer1).await;

    let peer2 = Peer::new("AliceUpdated".to_string(), addr);
    registry.upsert(peer2).await;

    let peers = registry.get_all().await;
    assert_eq!(peers.len(), 1);
    assert_eq!(peers[0].nickname, "AliceUpdated");
}

#[tokio::test]
async fn test_peer_timeout() {
    let registry = PeerRegistry::new();
    let addr = test_addr(8080);

    let mut peer = Peer::new("Alice".to_string(), addr);

    peer.last_seen = std::time::Instant::now() - TEST_TIMEOUT - Duration::from_secs(1);

    registry.upsert(peer).await;

    let removed = registry.remove_timed_out(TEST_TIMEOUT).await;

    assert_eq!(removed.len(), 1);
    assert_eq!(removed[0].nickname, "Alice");

    let peers = registry.get_all().await;
    assert_eq!(peers.len(), 0);
}

#[tokio::test]
async fn test_peer_not_timed_out() {
    let registry = PeerRegistry::new();
    let addr = test_addr(8080);
    let peer = Peer::new("Alice".to_string(), addr);

    registry.upsert(peer).await;

    let removed = registry.remove_timed_out(TEST_TIMEOUT).await;
    assert_eq!(removed.len(), 0);

    let peers = registry.get_all().await;
    assert_eq!(peers.len(), 1);
}

#[tokio::test]
async fn test_peer_get_by_id() {
    let registry = PeerRegistry::new();
    let addr = test_addr(8080);
    let peer = Peer::new("Alice".to_string(), addr);
    let peer_id = peer.id;

    registry.upsert(peer).await;

    let retrieved = registry.get(&peer_id).await;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().nickname, "Alice");
}

#[tokio::test]
async fn test_peer_remove() {
    let registry = PeerRegistry::new();
    let addr = test_addr(8080);
    let peer = Peer::new("Alice".to_string(), addr);
    let peer_id = peer.id;

    registry.upsert(peer).await;
    assert_eq!(registry.count().await, 1);

    registry.remove(&peer_id).await;
    assert_eq!(registry.count().await, 0);
}
