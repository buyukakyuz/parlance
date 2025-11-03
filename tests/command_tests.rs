//! Integration tests for command parsing.

use parlance::app::command::{Command, CommandParseError};

#[test]
fn test_parse_send() {
    let cmd = Command::parse("/send bob hello world").unwrap();
    assert_eq!(
        cmd,
        Command::Send {
            to: "bob".to_string(),
            content: "hello world".to_string()
        }
    );
}

#[test]
fn test_parse_send_multiword_message() {
    let cmd = Command::parse("/send alice this is a long message").unwrap();
    assert_eq!(
        cmd,
        Command::Send {
            to: "alice".to_string(),
            content: "this is a long message".to_string()
        }
    );
}

#[test]
fn test_parse_peers() {
    let cmd = Command::parse("/peers").unwrap();
    assert_eq!(cmd, Command::Peers);
}

#[test]
fn test_parse_quit_variants() {
    assert_eq!(Command::parse("/quit").unwrap(), Command::Quit);
    assert_eq!(Command::parse("/exit").unwrap(), Command::Quit);
    assert_eq!(Command::parse("/q").unwrap(), Command::Quit);
}

#[test]
fn test_parse_help_variants() {
    assert_eq!(Command::parse("/help").unwrap(), Command::Help);
    assert_eq!(Command::parse("/h").unwrap(), Command::Help);
}

#[test]
fn test_parse_unknown_command() {
    let result = Command::parse("/unknown");
    assert!(matches!(result, Err(CommandParseError::UnknownCommand(_))));
}

#[test]
fn test_parse_not_a_command() {
    let result = Command::parse("hello");
    assert!(matches!(result, Err(CommandParseError::NotACommand)));
}

#[test]
fn test_parse_send_missing_message() {
    let result = Command::parse("/send bob");
    assert!(matches!(
        result,
        Err(CommandParseError::MissingArguments { .. })
    ));
}

#[test]
fn test_parse_send_missing_all_args() {
    let result = Command::parse("/send");
    assert!(matches!(
        result,
        Err(CommandParseError::MissingArguments { .. })
    ));
}

#[test]
fn test_parse_with_extra_whitespace() {
    let cmd = Command::parse("  /peers  ").unwrap();
    assert_eq!(cmd, Command::Peers);
}

#[test]
fn test_help_text_not_empty() {
    let help = Command::help_text();
    assert!(!help.is_empty());
    assert!(help.contains("/send"));
    assert!(help.contains("/peers"));
    assert!(help.contains("/quit"));
    assert!(help.contains("/help"));
}
