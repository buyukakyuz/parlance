//! TCP messaging between peers.
//!
//! This module handles direct peer-to-peer messaging over TCP.
//! Each peer listens on a TCP port and can send/receive messages.

use crate::core::error::{ParlanceError, Result};
use crate::core::peer::PeerRegistry;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

/// A text message sent between peers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextMessage {
    /// Sender's nickname
    pub from: String,
    /// Message content
    pub content: String,
    /// Unix timestamp (seconds since epoch)
    pub timestamp: i64,
}

impl TextMessage {
    /// Create a new text message
    pub fn new(from: String, content: String) -> Self {
        Self {
            from,
            content,
            timestamp: Utc::now().timestamp(),
        }
    }

    /// Format the message for display
    pub fn format(&self) -> String {
        let datetime = chrono::DateTime::from_timestamp(self.timestamp, 0)
            .map(|dt| dt.format("%H:%M:%S").to_string())
            .unwrap_or_else(|| "??:??:??".to_string());

        format!("[{}] {}: {}", datetime, self.from, self.content)
    }
}

/// Events that occur in the messaging system
#[derive(Debug, Clone)]
pub enum MessageEvent {
    /// A message was received from a peer
    Received(TextMessage),
    /// A message was successfully sent to a peer
    #[allow(dead_code)]
    Sent { to: String, content: String },
    /// An error occurred while sending a message
    #[allow(dead_code)]
    SendError { to: String, error: String },
}

/// Messaging service configuration
pub struct MessagingConfig {
    /// Our nickname
    pub nickname: String,
    /// Port to listen on for incoming connections
    pub tcp_port: u16,
    /// Peer registry for looking up peers
    pub registry: PeerRegistry,
}

/// Messaging service
pub struct MessagingService {
    config: MessagingConfig,
    listener: TcpListener,
    event_tx: mpsc::UnboundedSender<MessageEvent>,
}

impl MessagingService {
    /// Create a new messaging service
    pub async fn new(
        config: MessagingConfig,
        event_tx: mpsc::UnboundedSender<MessageEvent>,
    ) -> Result<Self> {
        let bind_addr = SocketAddr::from(([0, 0, 0, 0], config.tcp_port));

        let listener =
            TcpListener::bind(bind_addr)
                .await
                .map_err(|e| ParlanceError::BindError {
                    address: bind_addr.to_string(),
                    source: e,
                })?;

        let local_addr = listener.local_addr()?;
        tracing::info!(addr = %local_addr, "Messaging service listening");

        Ok(Self {
            config,
            listener,
            event_tx,
        })
    }

    /// Get the local address the service is listening on
    pub fn local_addr(&self) -> Result<SocketAddr> {
        Ok(self.listener.local_addr()?)
    }

    /// Send a message to a peer by nickname
    pub async fn send_message(&self, to_nickname: &str, content: String) -> Result<()> {
        // Find the peer
        let peers = self.config.registry.get_all().await;
        let peer = peers
            .iter()
            .find(|p| p.nickname == to_nickname)
            .ok_or_else(|| ParlanceError::PeerNotFound(to_nickname.to_string()))?;

        // Connect to the peer
        let stream = TcpStream::connect(peer.addr).await.map_err(|e| {
            tracing::error!(
                peer = %to_nickname,
                addr = %peer.addr,
                error = ?e,
                "Failed to connect to peer"
            );
            e
        })?;

        // Create and send the message
        let msg = TextMessage::new(self.config.nickname.clone(), content.clone());
        let data = serde_json::to_string(&msg)?;

        let mut stream = stream;
        stream.write_all(data.as_bytes()).await?;
        stream.write_all(b"\n").await?;
        stream.flush().await?;

        tracing::info!(to = %to_nickname, "Message sent");

        // Notify that message was sent
        let _ = self.event_tx.send(MessageEvent::Sent {
            to: to_nickname.to_string(),
            content,
        });

        Ok(())
    }

    /// Handle an incoming TCP connection
    async fn handle_connection(
        stream: TcpStream,
        peer_addr: SocketAddr,
        event_tx: mpsc::UnboundedSender<MessageEvent>,
    ) {
        tracing::debug!(peer = %peer_addr, "New connection");

        let reader = BufReader::new(stream);
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            match serde_json::from_str::<TextMessage>(&line) {
                Ok(msg) => {
                    tracing::info!(
                        from = %msg.from,
                        content = %msg.content,
                        "Message received"
                    );

                    if event_tx.send(MessageEvent::Received(msg)).is_err() {
                        tracing::error!("Event channel closed");
                        break;
                    }
                }
                Err(e) => {
                    tracing::warn!(error = ?e, line = %line, "Invalid message format");
                }
            }
        }

        tracing::debug!(peer = %peer_addr, "Connection closed");
    }

    /// Run the messaging service
    ///
    /// This accepts incoming TCP connections and handles them concurrently.
    pub async fn run(&self) -> Result<()> {
        loop {
            match self.listener.accept().await {
                Ok((stream, peer_addr)) => {
                    let event_tx = self.event_tx.clone();
                    tokio::spawn(async move {
                        Self::handle_connection(stream, peer_addr, event_tx).await;
                    });
                }
                Err(e) => {
                    tracing::error!(error = ?e, "Failed to accept connection");
                }
            }
        }
    }
}

/// Helper to send a message to a peer
#[allow(dead_code)]
pub async fn send_to_peer(
    nickname: &str,
    to_nickname: &str,
    content: String,
    registry: &PeerRegistry,
) -> Result<()> {
    let peers = registry.get_all().await;
    let peer = peers
        .iter()
        .find(|p| p.nickname == to_nickname)
        .ok_or_else(|| ParlanceError::PeerNotFound(to_nickname.to_string()))?;

    let stream = TcpStream::connect(peer.addr).await?;

    let msg = TextMessage::new(nickname.to_string(), content);
    let data = serde_json::to_string(&msg)?;

    let mut stream = stream;
    stream.write_all(data.as_bytes()).await?;
    stream.write_all(b"\n").await?;
    stream.flush().await?;

    Ok(())
}
