//! Integration tests for messaging functionality.

use parlance::network::messaging::TextMessage;

#[test]
fn test_text_message_creation() {
    let msg = TextMessage::new("Alice".to_string(), "Hello!".to_string());

    assert_eq!(msg.from, "Alice");
    assert_eq!(msg.content, "Hello!");
    assert!(msg.timestamp > 0);
}

#[test]
fn test_text_message_format() {
    let msg = TextMessage::new("Alice".to_string(), "Hello World!".to_string());
    let formatted = msg.format();

    assert!(formatted.contains("Alice"));
    assert!(formatted.contains("Hello World!"));
    assert!(formatted.contains("["));
    assert!(formatted.contains("]"));
}

#[test]
fn test_text_message_serialization() {
    let msg = TextMessage::new("Alice".to_string(), "Test message".to_string());

    let json = serde_json::to_string(&msg).expect("Failed to serialize");

    let deserialized: TextMessage = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized.from, msg.from);
    assert_eq!(deserialized.content, msg.content);
    assert_eq!(deserialized.timestamp, msg.timestamp);
}

#[test]
fn test_text_message_with_special_characters() {
    let content = "Hello! 🦀 Special chars: @#$%^&*()";
    let msg = TextMessage::new("Bob".to_string(), content.to_string());

    assert_eq!(msg.content, content);

    let json = serde_json::to_string(&msg).expect("Failed to serialize");
    let deserialized: TextMessage = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized.content, content);
}

#[test]
fn test_text_message_empty_content() {
    let msg = TextMessage::new("Alice".to_string(), "".to_string());

    assert_eq!(msg.from, "Alice");
    assert_eq!(msg.content, "");
}

#[test]
fn test_text_message_long_content() {
    let long_content = "a".repeat(10000);
    let msg = TextMessage::new("Alice".to_string(), long_content.clone());

    assert_eq!(msg.content.len(), 10000);
    assert_eq!(msg.content, long_content);
}
