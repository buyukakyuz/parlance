//! Tests for nickname validation.

use parlance::core::validation::{NicknameValidationError, NicknameValidator};

#[test]
fn test_valid_nicknames() {
    assert!(NicknameValidator::validate("alice").is_ok());
    assert!(NicknameValidator::validate("Bob123").is_ok());
    assert!(NicknameValidator::validate("user-name_123").is_ok());
    assert!(NicknameValidator::validate("a").is_ok());
}

#[test]
fn test_empty_nickname() {
    let result = NicknameValidator::validate("");
    assert!(matches!(result, Err(NicknameValidationError::Empty)));
}

#[test]
fn test_whitespace_only() {
    let result = NicknameValidator::validate("   ");
    assert!(matches!(
        result,
        Err(NicknameValidationError::WhitespaceOnly)
    ));
}

#[test]
fn test_leading_trailing_whitespace() {
    assert!(matches!(
        NicknameValidator::validate(" alice"),
        Err(NicknameValidationError::LeadingOrTrailingWhitespace)
    ));

    assert!(matches!(
        NicknameValidator::validate("alice "),
        Err(NicknameValidationError::LeadingOrTrailingWhitespace)
    ));
}

#[test]
fn test_too_long() {
    let long_nickname = "a".repeat(33);
    let result = NicknameValidator::validate(&long_nickname);
    assert!(matches!(
        result,
        Err(NicknameValidationError::TooLong { .. })
    ));
}

#[test]
fn test_contains_newline() {
    assert!(matches!(
        NicknameValidator::validate("alice\nbob"),
        Err(NicknameValidationError::ContainsNewline)
    ));

    assert!(matches!(
        NicknameValidator::validate("alice\r\nbob"),
        Err(NicknameValidationError::ContainsNewline)
    ));
}

#[test]
fn test_invalid_characters() {
    // Control characters
    assert!(matches!(
        NicknameValidator::validate("alice\x00bob"),
        Err(NicknameValidationError::InvalidCharacters)
    ));
}

#[test]
fn test_max_length_boundary() {
    let exactly_max = "a".repeat(NicknameValidator::MAX_LENGTH);
    assert!(NicknameValidator::validate(&exactly_max).is_ok());

    let one_over_max = "a".repeat(NicknameValidator::MAX_LENGTH + 1);
    assert!(matches!(
        NicknameValidator::validate(&one_over_max),
        Err(NicknameValidationError::TooLong { .. })
    ));
}
