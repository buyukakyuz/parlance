//! Parlance Application
//!
//! A peer-to-peer messaging application for local networks using UDP multicast
//! for peer discovery and TCP for direct messaging.

mod app;
mod core;
mod network;

use app::{App, AppConfig};
use clap::Parser;
use core::error::Result;
use core::validation::NicknameValidator;
use std::path::PathBuf;
use tracing_subscriber::fmt;

/// Parlance - Local Network P2P Messaging
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Nickname (1-32 characters, no control characters)
    #[arg(short, long)]
    nickname: String,

    /// Path to configuration file (optional)
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Generate a default configuration file
    #[arg(long, value_name = "FILE")]
    generate_config: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    let args = Args::parse();

    if let Some(config_path) = args.generate_config {
        core::config::Config::write_default(&config_path)?;
        println!(
            "Generated default configuration at: {}",
            config_path.display()
        );
        return Ok(());
    }

    let config = if let Some(config_path) = args.config {
        core::config::Config::from_file(&config_path)
            .map_err(|e| core::error::ParlanceError::ConfigError(e.to_string()))?
    } else {
        core::config::Config::default()
    };

    NicknameValidator::validate(&args.nickname)
        .map_err(|e| core::error::ParlanceError::ConfigError(format!("Invalid nickname: {}", e)))?;

    let app_config = AppConfig::new(args.nickname);
    let app = App::new(app_config, config);

    app.run().await
}
