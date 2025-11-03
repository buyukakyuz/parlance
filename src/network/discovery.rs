//! Peer discovery via UDP multicast.
//!
//! This module implements automatic peer discovery on the local network
//! using UDP multicast. Peers broadcast their presence every 5 seconds
//! and listen for announcements from others.

use crate::core::error::{ParlanceError, Result};
use crate::core::peer::{Peer, PeerRegistry};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time;

/// Multicast group address for peer discovery
/// This is part of the Parlance protocol - all peers must use the same address
pub const MULTICAST_ADDR: Ipv4Addr = Ipv4Addr::new(239, 255, 255, 250);

/// Multicast port for peer discovery
/// This is part of the Parlance protocol - all peers must use the same port
pub const MULTICAST_PORT: u16 = 6789;

/// Discovery message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum DiscoveryMessage {
    /// Announce presence to other peers
    Announce { nickname: String, tcp_port: u16 },
    /// Goodbye message when shutting down
    Goodbye { nickname: String },
}

/// Discovery service configuration
pub struct DiscoveryConfig {
    /// Our nickname
    pub nickname: String,
    /// Our TCP port for messaging
    pub tcp_port: u16,
    /// Peer registry to update
    pub registry: PeerRegistry,
    /// Interval between announcements
    pub announce_interval: Duration,
    /// Peer timeout duration
    pub peer_timeout: Duration,
}

/// Discovery service handle
pub struct DiscoveryService {
    socket: UdpSocket,
    config: DiscoveryConfig,
    multicast_addr: SocketAddr,
}

impl DiscoveryService {
    /// Create a new discovery service
    pub async fn new(config: DiscoveryConfig) -> Result<Self> {
        let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), MULTICAST_PORT);

        // Create a socket with SO_REUSEADDR and SO_REUSEPORT enabled
        // This allows multiple instances to bind to the same multicast port
        let socket = socket2::Socket::new(
            socket2::Domain::IPV4,
            socket2::Type::DGRAM,
            Some(socket2::Protocol::UDP),
        )?;

        socket.set_reuse_address(true)?;

        // On Unix systems, also set SO_REUSEPORT to allow multiple binds
        #[cfg(all(unix, not(target_os = "solaris"), not(target_os = "illumos")))]
        {
            use std::os::unix::io::AsRawFd;
            let fd = socket.as_raw_fd();
            unsafe {
                let optval: libc::c_int = 1;
                libc::setsockopt(
                    fd,
                    libc::SOL_SOCKET,
                    libc::SO_REUSEPORT,
                    &optval as *const _ as *const libc::c_void,
                    std::mem::size_of_val(&optval) as libc::socklen_t,
                );
            }
        }

        socket.bind(&bind_addr.into())?;
        socket.set_nonblocking(true)?;

        let socket: std::net::UdpSocket = socket.into();
        let socket = UdpSocket::from_std(socket)?;

        // Join the multicast group
        socket
            .join_multicast_v4(MULTICAST_ADDR, Ipv4Addr::UNSPECIFIED)
            .map_err(|e| ParlanceError::MulticastJoinError {
                group: MULTICAST_ADDR.to_string(),
                source: e,
            })?;

        // Enable multicast loop so we can see our own messages (useful for debugging)
        socket.set_multicast_loop_v4(true)?;

        let multicast_addr = SocketAddr::new(IpAddr::V4(MULTICAST_ADDR), MULTICAST_PORT);

        tracing::info!(
            multicast_addr = %multicast_addr,
            "Discovery service started"
        );

        Ok(Self {
            socket,
            config,
            multicast_addr,
        })
    }

    /// Send an announcement to the multicast group
    #[allow(dead_code)]
    async fn announce(&self) -> Result<()> {
        let msg = DiscoveryMessage::Announce {
            nickname: self.config.nickname.clone(),
            tcp_port: self.config.tcp_port,
        };

        let data = serde_json::to_vec(&msg)?;
        self.socket.send_to(&data, self.multicast_addr).await?;

        tracing::debug!("Sent announcement");
        Ok(())
    }

    /// Send a goodbye message to the multicast group
    #[allow(dead_code)]
    pub async fn send_goodbye(&self) -> Result<()> {
        let msg = DiscoveryMessage::Goodbye {
            nickname: self.config.nickname.clone(),
        };

        let data = serde_json::to_vec(&msg)?;
        self.socket.send_to(&data, self.multicast_addr).await?;

        tracing::info!("Sent goodbye message");
        Ok(())
    }

    /// Handle a received discovery message
    #[allow(dead_code)]
    async fn handle_message(&self, data: &[u8], from: SocketAddr) -> Result<()> {
        let msg: DiscoveryMessage = serde_json::from_slice(data)?;

        match msg {
            DiscoveryMessage::Announce { nickname, tcp_port } => {
                // Don't add ourselves as a peer
                if nickname == self.config.nickname {
                    return Ok(());
                }

                // Create peer address using the sender's IP and their announced TCP port
                let peer_addr = SocketAddr::new(from.ip(), tcp_port);
                let peer = Peer::new(nickname, peer_addr);

                self.config.registry.upsert(peer).await;
            }
            DiscoveryMessage::Goodbye { nickname } => {
                tracing::info!(nickname = %nickname, "Received goodbye from peer");
                // Peer will be removed by timeout mechanism
            }
        }

        Ok(())
    }

    /// Run the discovery service
    ///
    /// This function runs two concurrent tasks:
    /// 1. Periodically announce our presence
    /// 2. Listen for announcements from other peers
    pub async fn run(self) -> Result<()> {
        let socket = std::sync::Arc::new(self.socket);
        let config = std::sync::Arc::new(self.config);

        // Task 1: Periodic announcements
        let announce_socket = socket.clone();
        let announce_config = config.clone();
        let multicast_addr = self.multicast_addr;

        let announce_task = tokio::spawn(async move {
            let mut interval = time::interval(announce_config.announce_interval);
            loop {
                interval.tick().await;

                let msg = DiscoveryMessage::Announce {
                    nickname: announce_config.nickname.clone(),
                    tcp_port: announce_config.tcp_port,
                };

                match serde_json::to_vec(&msg) {
                    Ok(data) => {
                        if let Err(e) = announce_socket.send_to(&data, multicast_addr).await {
                            tracing::error!(error = ?e, "Failed to send announcement");
                        } else {
                            tracing::debug!("Sent announcement");
                        }
                    }
                    Err(e) => {
                        tracing::error!(error = ?e, "Failed to serialize announcement");
                    }
                }
            }
        });

        // Task 2: Listen for messages
        let listen_socket = socket.clone();
        let listen_config = config.clone();

        let listen_task = tokio::spawn(async move {
            let mut buf = vec![0u8; 65536];
            loop {
                match listen_socket.recv_from(&mut buf).await {
                    Ok((len, from)) => {
                        let data = &buf[..len];

                        match serde_json::from_slice::<DiscoveryMessage>(data) {
                            Ok(msg) => {
                                match msg {
                                    DiscoveryMessage::Announce { nickname, tcp_port } => {
                                        // Don't add ourselves
                                        if nickname == listen_config.nickname {
                                            continue;
                                        }

                                        let peer_addr = SocketAddr::new(from.ip(), tcp_port);
                                        let peer = Peer::new(nickname, peer_addr);
                                        listen_config.registry.upsert(peer).await;
                                    }
                                    DiscoveryMessage::Goodbye { nickname } => {
                                        tracing::info!(nickname = %nickname, "Received goodbye");
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::warn!(error = ?e, "Failed to parse discovery message");
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!(error = ?e, "Failed to receive on discovery socket");
                    }
                }
            }
        });

        // Task 3: Cleanup timed-out peers
        let cleanup_registry = config.registry.clone();
        let cleanup_timeout = config.peer_timeout;
        let cleanup_task = tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(5));
            loop {
                interval.tick().await;
                cleanup_registry.remove_timed_out(cleanup_timeout).await;
            }
        });

        // Run all tasks
        tokio::select! {
            _ = announce_task => {
                tracing::error!("Announce task terminated unexpectedly");
            }
            _ = listen_task => {
                tracing::error!("Listen task terminated unexpectedly");
            }
            _ = cleanup_task => {
                tracing::error!("Cleanup task terminated unexpectedly");
            }
        }

        Ok(())
    }
}
