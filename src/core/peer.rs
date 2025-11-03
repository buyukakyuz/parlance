//! Peer management module.
//!
//! This module handles peer representation and the peer registry,
//! which tracks all discovered peers on the local network.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Unique identifier for a peer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerId(Uuid);

impl PeerId {
    /// Create a new random peer ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create a peer ID from a socket address (deterministic)
    pub fn from_addr(addr: &SocketAddr) -> Self {
        let hash = format!("{}", addr);
        Self(Uuid::new_v5(&Uuid::NAMESPACE_DNS, hash.as_bytes()))
    }
}

impl Default for PeerId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for PeerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0.to_string()[..8])
    }
}

/// Represents a peer on the network
#[derive(Debug, Clone)]
pub struct Peer {
    /// Unique identifier
    pub id: PeerId,
    /// User-chosen nickname
    pub nickname: String,
    /// Socket address for TCP connections
    pub addr: SocketAddr,
    /// Last time we received an announcement from this peer
    pub last_seen: Instant,
}

impl Peer {
    /// Create a new peer
    pub fn new(nickname: String, addr: SocketAddr) -> Self {
        Self {
            id: PeerId::from_addr(&addr),
            nickname,
            addr,
            last_seen: Instant::now(),
        }
    }

    /// Update the last_seen timestamp
    pub fn refresh(&mut self) {
        self.last_seen = Instant::now();
    }

    /// Check if this peer has timed out
    pub fn is_timed_out(&self, timeout: Duration) -> bool {
        self.last_seen.elapsed() > timeout
    }
}

/// Thread-safe peer registry
///
/// Maintains a list of all discovered peers and provides methods
/// to add, update, and remove peers based on timeouts.
#[derive(Clone)]
pub struct PeerRegistry {
    peers: Arc<RwLock<HashMap<PeerId, Peer>>>,
}

impl PeerRegistry {
    /// Create a new empty peer registry
    pub fn new() -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add or update a peer in the registry
    pub async fn upsert(&self, peer: Peer) {
        let mut peers = self.peers.write().await;
        if let Some(existing) = peers.get_mut(&peer.id) {
            existing.refresh();
            existing.nickname = peer.nickname;
            existing.addr = peer.addr;
        } else {
            tracing::info!(
                peer_id = %peer.id,
                nickname = %peer.nickname,
                addr = %peer.addr,
                "New peer discovered"
            );
            peers.insert(peer.id, peer);
        }
    }

    /// Remove a peer by ID
    #[allow(dead_code)]
    pub async fn remove(&self, id: &PeerId) {
        let mut peers = self.peers.write().await;
        if let Some(peer) = peers.remove(id) {
            tracing::info!(
                peer_id = %peer.id,
                nickname = %peer.nickname,
                "Peer removed"
            );
        }
    }

    /// Remove all timed-out peers
    pub async fn remove_timed_out(&self, timeout: Duration) -> Vec<Peer> {
        let mut peers = self.peers.write().await;
        let timed_out: Vec<_> = peers
            .iter()
            .filter(|(_, p)| p.is_timed_out(timeout))
            .map(|(id, _)| *id)
            .collect();

        let mut removed = Vec::new();
        for id in timed_out {
            if let Some(peer) = peers.remove(&id) {
                tracing::warn!(
                    peer_id = %peer.id,
                    nickname = %peer.nickname,
                    "Peer timed out"
                );
                removed.push(peer);
            }
        }
        removed
    }

    /// Get all active peers
    pub async fn get_all(&self) -> Vec<Peer> {
        let peers = self.peers.read().await;
        peers.values().cloned().collect()
    }

    /// Get a peer by ID
    #[allow(dead_code)]
    pub async fn get(&self, id: &PeerId) -> Option<Peer> {
        let peers = self.peers.read().await;
        peers.get(id).cloned()
    }

    /// Get the number of active peers
    #[allow(dead_code)]
    pub async fn count(&self) -> usize {
        let peers = self.peers.read().await;
        peers.len()
    }
}

impl Default for PeerRegistry {
    fn default() -> Self {
        Self::new()
    }
}
