//! Input validation utilities.

/// Nickname validation rules
pub struct NicknameValidator;

impl NicknameValidator {
    /// Minimum nickname length
    pub const MIN_LENGTH: usize = 1;

    /// Maximum nickname length
    pub const MAX_LENGTH: usize = 32;

    /// Validate a nickname
    pub fn validate(nickname: &str) -> Result<(), NicknameValidationError> {
        if nickname.is_empty() {
            return Err(NicknameValidationError::Empty);
        }

        if nickname.len() < Self::MIN_LENGTH {
            return Err(NicknameValidationError::TooShort {
                min: Self::MIN_LENGTH,
            });
        }

        if nickname.len() > Self::MAX_LENGTH {
            return Err(NicknameValidationError::TooLong {
                max: Self::MAX_LENGTH,
                actual: nickname.len(),
            });
        }

        if nickname.trim().is_empty() {
            return Err(NicknameValidationError::WhitespaceOnly);
        }

        if nickname != nickname.trim() {
            return Err(NicknameValidationError::LeadingOrTrailingWhitespace);
        }

        // Check for newlines first (would break protocol)
        // Must come before control character check since newlines are control chars
        if nickname.contains('\n') || nickname.contains('\r') {
            return Err(NicknameValidationError::ContainsNewline);
        }

        // Check for other invalid characters (control characters, etc.)
        if nickname.chars().any(|c| c.is_control()) {
            return Err(NicknameValidationError::InvalidCharacters);
        }

        Ok(())
    }
}

/// Nickname validation errors
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NicknameValidationError {
    #[error("Nickname cannot be empty")]
    Empty,

    #[error("Nickname must be at least {min} characters long")]
    TooShort { min: usize },

    #[error("Nickname must be at most {max} characters long (got {actual})")]
    TooLong { max: usize, actual: usize },

    #[error("Nickname cannot be only whitespace")]
    WhitespaceOnly,

    #[error("Nickname cannot have leading or trailing whitespace")]
    LeadingOrTrailingWhitespace,

    #[error("Nickname contains invalid characters")]
    InvalidCharacters,

    #[error("Nickname cannot contain newlines")]
    ContainsNewline,
}
