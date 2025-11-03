//! Application configuration.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::Duration;

/// Peer behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerConfig {
    /// Timeout in seconds before a peer is considered offline
    /// Default: 15 seconds
    #[serde(default = "default_peer_timeout_secs")]
    pub timeout_secs: u64,

    /// Interval in seconds between peer announcements
    /// Default: 5 seconds
    #[serde(default = "default_announce_interval_secs")]
    pub announce_interval_secs: u64,
}

/// Complete application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub peer: PeerConfig,
}

impl Config {
    /// Load configuration from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let contents = fs::read_to_string(path.as_ref()).map_err(|e| ConfigError::IoError {
            path: path.as_ref().display().to_string(),
            source: e,
        })?;

        toml::from_str(&contents).map_err(|e| ConfigError::ParseError {
            path: path.as_ref().display().to_string(),
            source: e,
        })
    }

    /// Get peer timeout as Duration
    pub fn peer_timeout(&self) -> Duration {
        Duration::from_secs(self.peer.timeout_secs)
    }

    /// Get announcement interval as Duration
    pub fn announce_interval(&self) -> Duration {
        Duration::from_secs(self.peer.announce_interval_secs)
    }

    /// Create a default configuration and write it to a file
    pub fn write_default<P: AsRef<Path>>(path: P) -> Result<(), ConfigError> {
        let config = Config::default();
        let toml = toml::to_string_pretty(&config)
            .map_err(|e| ConfigError::SerializeError { source: e })?;

        fs::write(path.as_ref(), toml).map_err(|e| ConfigError::IoError {
            path: path.as_ref().display().to_string(),
            source: e,
        })?;

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            peer: PeerConfig::default(),
        }
    }
}

impl Default for PeerConfig {
    fn default() -> Self {
        Self {
            timeout_secs: default_peer_timeout_secs(),
            announce_interval_secs: default_announce_interval_secs(),
        }
    }
}

// Default value functions for serde
fn default_peer_timeout_secs() -> u64 {
    15
}

fn default_announce_interval_secs() -> u64 {
    5
}

/// Configuration errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to read config file {path}: {source}")]
    IoError {
        path: String,
        source: std::io::Error,
    },

    #[error("Failed to parse config file {path}: {source}")]
    ParseError {
        path: String,
        source: toml::de::Error,
    },

    #[error("Failed to serialize config: {source}")]
    SerializeError { source: toml::ser::Error },
}
