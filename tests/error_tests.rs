//! Integration tests for error handling.

use parlance::core::error::ParlanceError;
use std::io;

#[test]
fn test_peer_not_found_error() {
    let err = ParlanceError::PeerNotFound("Alice".to_string());
    let error_msg = err.to_string();

    assert!(error_msg.contains("Peer not found"));
    assert!(error_msg.contains("Alice"));
}

#[test]
fn test_bind_error() {
    let io_err = io::Error::new(io::ErrorKind::AddrInUse, "Address in use");
    let err = ParlanceError::BindError {
        address: "0.0.0.0:8080".to_string(),
        source: io_err,
    };

    let error_msg = err.to_string();
    assert!(error_msg.contains("Failed to bind"));
    assert!(error_msg.contains("0.0.0.0:8080"));
}

#[test]
fn test_multicast_join_error() {
    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "Permission denied");
    let err = ParlanceError::MulticastJoinError {
        group: "239.255.255.250".to_string(),
        source: io_err,
    };

    let error_msg = err.to_string();
    assert!(error_msg.contains("Failed to join multicast group"));
    assert!(error_msg.contains("239.255.255.250"));
}

#[test]
fn test_channel_send_error() {
    let err = ParlanceError::ChannelSendError;
    let error_msg = err.to_string();

    assert!(error_msg.contains("Channel send error"));
    assert!(error_msg.contains("channel closed"));
}

#[test]
fn test_config_error() {
    let err = ParlanceError::ConfigError("Invalid configuration".to_string());
    let error_msg = err.to_string();

    assert!(error_msg.contains("Configuration error"));
    assert!(error_msg.contains("Invalid configuration"));
}

#[test]
fn test_serialization_error() {
    let invalid_json = "{invalid json}";
    let result: Result<serde_json::Value, _> = serde_json::from_str(invalid_json);

    assert!(result.is_err());

    let err = result.map_err(ParlanceError::from).unwrap_err();
    let error_msg = err.to_string();

    assert!(error_msg.contains("Serialization error"));
}

#[test]
fn test_network_error_from_io() {
    let io_err = io::Error::new(io::ErrorKind::ConnectionRefused, "Connection refused");
    let err = ParlanceError::Network(io_err);

    let error_msg = err.to_string();
    assert!(error_msg.contains("Network error"));
}

#[test]
fn test_error_debug_format() {
    let err = ParlanceError::PeerNotFound("TestPeer".to_string());

    let debug_str = format!("{:?}", err);
    assert!(debug_str.contains("PeerNotFound"));
    assert!(debug_str.contains("TestPeer"));
}
