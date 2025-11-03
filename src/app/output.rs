//! Output abstraction for user-facing messages.
//!
//! Centralizes all user output to make it easier to test and potentially
//! redirect output (e.g., to a GUI or different terminal).

use std::io::{self, Write};

/// Output interface for user messages
pub struct Output;

impl Output {
    /// Print a regular informational message
    pub fn info(message: &str) {
        println!("{}", message);
    }

    /// Print a success message
    pub fn success(message: &str) {
        println!("{}", message);
    }

    /// Print an error message
    pub fn error(message: &str) {
        eprintln!("{}", message);
    }

    #[allow(dead_code)]
    /// Print a warning message
    pub fn warning(message: &str) {
        println!("{}", message);
    }

    /// Print without a newline and flush
    pub fn prompt(message: &str) {
        print!("{}", message);
        let _ = io::stdout().flush();
    }

    /// Print a message received from a peer
    pub fn message_received(formatted: &str) {
        println!("\n{}", formatted);
        Self::prompt("> ");
    }

    /// Print the welcome banner
    pub fn welcome_banner(nickname: &str, tcp_port: u16) {
        println!("\n╔═══════════════════════════════════════╗");
        println!("║         Parlance Started!             ║");
        println!("╚═══════════════════════════════════════╝");
        println!();
        println!("Nickname: {}", nickname);
        println!("TCP Port: {}", tcp_port);
        println!();
        println!("Type /help for available commands");
        println!();
        println!("Waiting for peers...");
        println!();
    }

    /// Print the peer list
    pub fn peer_list(peers: &[(String, String)]) {
        println!("\n╔═══════════════════════════════════════╗");
        println!("║     Discovered Peers ({:2})             ║", peers.len());
        println!("╚═══════════════════════════════════════╝");

        if peers.is_empty() {
            println!("  No peers found yet...");
        } else {
            for (nickname, addr) in peers {
                println!("  • {} ({})", nickname, addr);
            }
        }
        println!();
    }
}
