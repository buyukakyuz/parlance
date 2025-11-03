//! Custom error types for Parlance application.
//!
//! This module defines all error types using `thiserror` for proper error handling
//! throughout the application. No `.unwrap()` or `.expect()` calls should be used
//! in production code; instead, errors should be propagated using `?`.

use super::config::ConfigError;
use std::io;

/// Main error type for Parlance application.
///
/// This enum covers all possible error conditions that can occur during
/// peer discovery, messaging, and network operations.
#[derive(Debug, thiserror::Error)]
pub enum ParlanceError {
    /// Network I/O error occurred
    #[error("Network error: {0}")]
    Network(#[from] io::Error),

    /// JSON serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Peer with the given ID was not found
    #[error("Peer not found: {0}")]
    PeerNotFound(String),

    /// Failed to bind to a network address
    #[error("Failed to bind to {address}: {source}")]
    BindError { address: String, source: io::Error },

    /// Failed to join multicast group
    #[error("Failed to join multicast group {group}: {source}")]
    MulticastJoinError { group: String, source: io::Error },

    /// Invalid message format received
    #[error("Invalid message format: {0}")]
    #[allow(dead_code)]
    InvalidMessage(String),

    /// Channel send error
    #[error("Channel send error: channel closed")]
    ChannelSendError,

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// UTF-8 conversion error
    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

/// Convenience type alias for Results using our custom error type.
pub type Result<T> = std::result::Result<T, ParlanceError>;

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for ParlanceError {
    fn from(_: tokio::sync::mpsc::error::SendError<T>) -> Self {
        ParlanceError::ChannelSendError
    }
}

impl From<ConfigError> for ParlanceError {
    fn from(e: ConfigError) -> Self {
        ParlanceError::ConfigError(e.to_string())
    }
}
