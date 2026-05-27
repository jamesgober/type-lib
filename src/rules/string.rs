//! String-content rules.
//!
//! Each rule applies to any `S: AsRef<str>`, so it accepts `&str`, `String`, and
//! borrowed string types alike. They check character content only; combine with
//! the [length rules](super::length) (via [`And`](crate::combinator::And)) when
//! you also need a length bound.

use crate::{ValidationError, Validator};

/// Accepts strings whose characters are all ASCII.
///
/// An empty string passes (it has no non-ASCII characters). Combine with
/// [`NonEmpty`](super::NonEmpty) if you also require content.
///
/// # Examples
///
/// ```rust
/// use type_lib::rules::Ascii;
/// use type_lib::Validator;
///
/// assert!(Ascii::validate("plain-text_123").is_ok());
/// assert!(Ascii::validate("café").is_err());
/// ```
pub struct Ascii;

impl<S: AsRef<str> + ?Sized> Validator<S> for Ascii {
    type Error = ValidationError;

    fn validate(value: &S) -> Result<(), Self::Error> {
        if value.as_ref().is_ascii() {
            Ok(())
        } else {
            Err(ValidationError::new(
                "ascii",
                "value must contain only ASCII characters",
            ))
        }
    }
}

/// Accepts strings whose characters are all alphanumeric.
///
/// Uses the Unicode definition of alphanumeric ([`char::is_alphanumeric`]). An
/// empty string passes vacuously; combine with [`NonEmpty`](super::NonEmpty) to
/// also require content.
///
/// # Examples
///
/// ```rust
/// use type_lib::rules::Alphanumeric;
/// use type_lib::Validator;
///
/// assert!(Alphanumeric::validate("abc123").is_ok());
/// assert!(Alphanumeric::validate("abc 123").is_err()); // space
/// assert!(Alphanumeric::validate("user_name").is_err()); // underscore
/// ```
pub struct Alphanumeric;

impl<S: AsRef<str> + ?Sized> Validator<S> for Alphanumeric {
    type Error = ValidationError;

    fn validate(value: &S) -> Result<(), Self::Error> {
        if value.as_ref().chars().all(char::is_alphanumeric) {
            Ok(())
        } else {
            Err(ValidationError::new(
                "alphanumeric",
                "value must contain only alphanumeric characters",
            ))
        }
    }
}

/// Accepts strings with no leading or trailing whitespace.
///
/// Useful for rejecting un-normalized input before it is stored. Whitespace is
/// defined by [`char::is_whitespace`] via [`str::trim`].
///
/// # Examples
///
/// ```rust
/// use type_lib::rules::Trimmed;
/// use type_lib::Validator;
///
/// assert!(Trimmed::validate("clean").is_ok());
/// assert!(Trimmed::validate(" padded ").is_err());
/// assert!(Trimmed::validate("trailing\n").is_err());
/// ```
pub struct Trimmed;

impl<S: AsRef<str> + ?Sized> Validator<S> for Trimmed {
    type Error = ValidationError;

    fn validate(value: &S) -> Result<(), Self::Error> {
        let value = value.as_ref();
        if value == value.trim() {
            Ok(())
        } else {
            Err(ValidationError::new(
                "trimmed",
                "value must not have leading or trailing whitespace",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn ascii_rule() {
        assert!(Ascii::validate("abc123!").is_ok());
        assert!(Ascii::validate("").is_ok());
        assert_eq!(Ascii::validate("é").unwrap_err().code(), "ascii");
    }

    #[test]
    fn alphanumeric_rule() {
        assert!(Alphanumeric::validate("Abc123").is_ok());
        assert!(Alphanumeric::validate("").is_ok());
        assert!(Alphanumeric::validate("a-b").is_err());
    }

    #[test]
    fn trimmed_rule() {
        assert!(Trimmed::validate("ok").is_ok());
        assert!(Trimmed::validate(" leading").is_err());
        assert!(Trimmed::validate("trailing ").is_err());
        assert!(Trimmed::validate("inner space ok".trim()).is_ok());
    }
}
