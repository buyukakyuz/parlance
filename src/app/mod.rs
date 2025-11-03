//! Application orchestration and runtime.
//!
//! This module contains the main application logic that coordinates
//! the discovery service, messaging service, and user input handling.

pub mod command;
pub mod output;

use command::Command;
use output::Output;

use crate::core::config::Config;
use crate::core::error::Result;
use crate::core::peer::PeerRegistry;
use crate::network::discovery::{DiscoveryConfig, DiscoveryService};
use crate::network::messaging::{MessageEvent, MessagingConfig, MessagingService};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::signal;
use tokio::sync::mpsc;
use tracing::{error, info};

/// Application configuration
pub struct AppConfig {
    pub nickname: String,
    pub tcp_port: u16,
}

impl AppConfig {
    /// Create a new config with a nickname and dynamic port
    pub fn new(nickname: String) -> Self {
        Self {
            nickname,
            tcp_port: 0, // Let OS assign port
        }
    }
}

/// Main application runtime
pub struct App {
    app_config: AppConfig,
    config: Config,
    registry: PeerRegistry,
}

impl App {
    /// Create a new application instance
    pub fn new(app_config: AppConfig, config: Config) -> Self {
        Self {
            app_config,
            config,
            registry: PeerRegistry::new(),
        }
    }

    /// Run the application
    pub async fn run(self) -> Result<()> {
        info!(nickname = %self.app_config.nickname, "Starting Parlance");

        // Create message event channel
        let (event_tx, event_rx) = mpsc::unbounded_channel::<MessageEvent>();

        // Create messaging service first to get the actual port
        let messaging_config = MessagingConfig {
            nickname: self.app_config.nickname.clone(),
            tcp_port: self.app_config.tcp_port,
            registry: self.registry.clone(),
        };

        let messaging_service = MessagingService::new(messaging_config, event_tx.clone()).await?;

        // Get the actual TCP port that was bound
        let actual_tcp_port = messaging_service.local_addr()?.port();
        info!(tcp_port = actual_tcp_port, "TCP port bound");

        // Create discovery service with the actual TCP port and config
        let discovery_config = DiscoveryConfig {
            nickname: self.app_config.nickname.clone(),
            tcp_port: actual_tcp_port,
            registry: self.registry.clone(),
            announce_interval: self.config.announce_interval(),
            peer_timeout: self.config.peer_timeout(),
        };

        let discovery_service = DiscoveryService::new(discovery_config).await?;

        Output::welcome_banner(&self.app_config.nickname, actual_tcp_port);

        // Spawn services and tasks
        let discovery_task = tokio::spawn(async move {
            if let Err(e) = discovery_service.run().await {
                error!(error = ?e, "Discovery service error");
            }
        });

        let msg_service = Arc::new(messaging_service);
        let msg_service_for_task = msg_service.clone();

        let messaging_task = tokio::spawn(async move {
            if let Err(e) = msg_service_for_task.run().await {
                error!(error = ?e, "Messaging service error");
            }
        });

        let input_task = self.spawn_input_handler(msg_service.clone());

        let event_task = Self::spawn_event_handler(event_rx);

        tokio::select! {
            _ = signal::ctrl_c() => {
                info!("Received Ctrl+C, shutting down...");
            }
            _ = input_task => {
                info!("Input task completed");
            }
        }

        Output::info("\nShutting down...");

        discovery_task.abort();
        messaging_task.abort();
        event_task.abort();

        Output::info("Goodbye!");

        Ok(())
    }

    /// Spawn the input handler task
    fn spawn_input_handler(
        &self,
        msg_service: Arc<MessagingService>,
    ) -> tokio::task::JoinHandle<()> {
        let registry = self.registry.clone();

        tokio::spawn(async move {
            let stdin = tokio::io::stdin();
            let reader = BufReader::new(stdin);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                let line = line.trim();

                if line.is_empty() {
                    continue;
                }

                match Command::parse(line) {
                    Ok(Command::Send { to, content }) => {
                        match msg_service.send_message(&to, content).await {
                            Ok(_) => {
                                Output::success(&format!("Message sent to {}", to));
                            }
                            Err(e) => {
                                Output::error(&format!("Failed to send message: {}", e));
                            }
                        }
                    }
                    Ok(Command::Peers) => {
                        Self::handle_peers_command(&registry).await;
                    }
                    Ok(Command::Quit) => {
                        info!("User requested quit");
                        break;
                    }
                    Ok(Command::Help) => {
                        Output::info(&format!("\n{}", Command::help_text()));
                    }
                    Err(e) => {
                        Output::error(&e.to_string());
                        if matches!(e, command::CommandParseError::UnknownCommand(_)) {
                            Output::info("Type /help for available commands");
                        }
                    }
                }
            }

            info!("Input handler exiting");
        })
    }

    /// Handle the /peers command
    async fn handle_peers_command(registry: &PeerRegistry) {
        let peers = registry.get_all().await;
        let peer_list: Vec<(String, String)> = peers
            .into_iter()
            .map(|p| (p.nickname, p.addr.to_string()))
            .collect();

        Output::peer_list(&peer_list);
    }

    /// Spawn the event handler task
    fn spawn_event_handler(
        mut event_rx: mpsc::UnboundedReceiver<MessageEvent>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                match event {
                    MessageEvent::Received(msg) => {
                        Output::message_received(&msg.format());
                    }
                    MessageEvent::Sent { to, content: _ } => {
                        // Already handled in input task
                        tracing::debug!(to = %to, "Message sent event");
                    }
                    MessageEvent::SendError { to, error } => {
                        Output::error(&format!("Error sending to {}: {}", to, error));
                        Output::prompt("> ");
                    }
                }
            }
        })
    }
}
