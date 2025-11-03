//! Integration tests for discovery protocol.

use parlance::network::discovery::DiscoveryMessage;

#[test]
fn test_announce_message_serialization() {
    let msg = DiscoveryMessage::Announce {
        nickname: "Alice".to_string(),
        tcp_port: 8080,
    };

    let json = serde_json::to_string(&msg).expect("Failed to serialize");

    assert!(json.contains("announce"));
    assert!(json.contains("Alice"));
    assert!(json.contains("8080"));
}

#[test]
fn test_announce_message_deserialization() {
    let json = r#"{"type":"announce","nickname":"Bob","tcp_port":9090}"#;

    let msg: DiscoveryMessage = serde_json::from_str(json).expect("Failed to deserialize");

    match msg {
        DiscoveryMessage::Announce { nickname, tcp_port } => {
            assert_eq!(nickname, "Bob");
            assert_eq!(tcp_port, 9090);
        }
        _ => panic!("Wrong message type"),
    }
}

#[test]
fn test_goodbye_message_serialization() {
    let msg = DiscoveryMessage::Goodbye {
        nickname: "Charlie".to_string(),
    };

    let json = serde_json::to_string(&msg).expect("Failed to serialize");

    assert!(json.contains("goodbye"));
    assert!(json.contains("Charlie"));
}

#[test]
fn test_goodbye_message_deserialization() {
    let json = r#"{"type":"goodbye","nickname":"Dave"}"#;

    let msg: DiscoveryMessage = serde_json::from_str(json).expect("Failed to deserialize");

    match msg {
        DiscoveryMessage::Goodbye { nickname } => {
            assert_eq!(nickname, "Dave");
        }
        _ => panic!("Wrong message type"),
    }
}

#[test]
fn test_discovery_message_roundtrip() {
    let original = DiscoveryMessage::Announce {
        nickname: "TestUser".to_string(),
        tcp_port: 12345,
    };

    let json = serde_json::to_string(&original).expect("Failed to serialize");
    let deserialized: DiscoveryMessage =
        serde_json::from_str(&json).expect("Failed to deserialize");

    match deserialized {
        DiscoveryMessage::Announce { nickname, tcp_port } => {
            assert_eq!(nickname, "TestUser");
            assert_eq!(tcp_port, 12345);
        }
        _ => panic!("Wrong message type after roundtrip"),
    }
}

#[test]
fn test_discovery_message_with_special_nickname() {
    let msg = DiscoveryMessage::Announce {
        nickname: "User-123_Test".to_string(),
        tcp_port: 5000,
    };

    let json = serde_json::to_string(&msg).expect("Failed to serialize");
    let deserialized: DiscoveryMessage =
        serde_json::from_str(&json).expect("Failed to deserialize");

    match deserialized {
        DiscoveryMessage::Announce { nickname, .. } => {
            assert_eq!(nickname, "User-123_Test");
        }
        _ => panic!("Wrong message type"),
    }
}
