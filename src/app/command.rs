//! Command parsing and representation.

use std::fmt;

/// User commands
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Send a message to a peer
    Send { to: String, content: String },
    /// List discovered peers
    Peers,
    /// Quit the application
    Quit,
    /// Display help
    Help,
}

/// Errors that can occur during command parsing
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandParseError {
    /// Command not recognized
    UnknownCommand(String),
    /// Command is missing required arguments
    MissingArguments { command: String, usage: String },
    /// Input doesn't start with command prefix
    NotACommand,
}

impl fmt::Display for CommandParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandParseError::UnknownCommand(cmd) => write!(f, "Unknown command: {}", cmd),
            CommandParseError::MissingArguments { command, usage } => {
                write!(f, "Usage: {} {}", command, usage)
            }
            CommandParseError::NotACommand => write!(f, "Commands must start with /"),
        }
    }
}

impl Command {
    /// Parse a command from user input
    pub fn parse(input: &str) -> Result<Self, CommandParseError> {
        let input = input.trim();

        if !input.starts_with('/') {
            return Err(CommandParseError::NotACommand);
        }

        let parts: Vec<&str> = input[1..].splitn(2, ' ').collect();
        let cmd = parts[0];

        match cmd {
            "send" => {
                if parts.len() < 2 {
                    return Err(CommandParseError::MissingArguments {
                        command: "/send".to_string(),
                        usage: "<nickname> <message>".to_string(),
                    });
                }

                let rest = parts[1];
                if let Some((to, content)) = rest.split_once(' ') {
                    Ok(Command::Send {
                        to: to.to_string(),
                        content: content.to_string(),
                    })
                } else {
                    Err(CommandParseError::MissingArguments {
                        command: "/send".to_string(),
                        usage: "<nickname> <message>".to_string(),
                    })
                }
            }
            "peers" => Ok(Command::Peers),
            "quit" | "exit" | "q" => Ok(Command::Quit),
            "help" | "h" => Ok(Command::Help),
            unknown => Err(CommandParseError::UnknownCommand(unknown.to_string())),
        }
    }

    /// Get help text for a command
    pub fn help_text() -> &'static str {
        r#"Available commands:
  /send <nickname> <message>  Send a message to a peer
  /peers                      List discovered peers
  /quit                       Exit the application
  /help                       Show this help"#
    }
}
