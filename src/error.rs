//! The crate's ready-made validation error.
//!
//! Most validators need to report only two things: *which* invariant failed and
//! a short human-readable explanation. [`ValidationError`] carries exactly that,
//! using two `'static` string slices so it allocates nothing and works under
//! `no_std`. Validators that need richer, structured failures are free to define
//! their own error type — [`Validator::Error`](crate::Validator::Error) is an
//! associated type, not a fixed choice.

use core::fmt;

/// A lightweight validation failure with a machine-readable code and a
/// human-readable message.
///
/// `ValidationError` is the default error returned by the simple validators in
/// this crate and the recommended starting point for hand-written ones. It is
/// `Copy`, holds no owned data, and never allocates, which keeps it usable on a
/// hot path and in `no_std` builds alike.
///
/// The `code` is meant to be a stable, lowercase identifier you can match on
/// (`"non_empty"`, `"out_of_range"`); the `message` is meant for logs and
/// end-user diagnostics. Keep the code stable across releases even if you reword
/// the message.
///
/// Under the `std` feature it implements [`std::error::Error`], so it slots into
/// `?` and `Box<dyn Error>` chains without ceremony.
///
/// # Examples
///
/// Construct one directly and inspect its parts:
///
/// ```rust
/// use type_lib::ValidationError;
///
/// let err = ValidationError::new("non_empty", "value must not be empty");
/// assert_eq!(err.code(), "non_empty");
/// assert_eq!(err.message(), "value must not be empty");
/// ```
///
/// Branch on the stable code while showing the message to a human:
///
/// ```rust
/// use type_lib::ValidationError;
///
/// fn describe(err: &ValidationError) -> &'static str {
///     match err.code() {
///         "non_empty" => "the field was left blank",
///         "out_of_range" => "the number is outside the allowed range",
///         _ => "the value is invalid",
///     }
/// }
///
/// let err = ValidationError::new("out_of_range", "expected 1..=10, got 42");
/// assert_eq!(describe(&err), "the number is outside the allowed range");
/// ```
///
/// Format it for display (the `Display` output is `"<code>: <message>"`):
///
/// ```rust
/// use type_lib::ValidationError;
///
/// let err = ValidationError::new("non_empty", "value must not be empty");
/// assert_eq!(err.to_string(), "non_empty: value must not be empty");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValidationError {
    code: &'static str,
    message: &'static str,
}

impl ValidationError {
    /// Creates a validation error from a stable `code` and a human-readable
    /// `message`.
    ///
    /// This is a `const fn`, so errors can be declared as associated constants
    /// or `static`s and reused without per-call construction cost.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use type_lib::ValidationError;
    ///
    /// const EMPTY: ValidationError =
    ///     ValidationError::new("non_empty", "value must not be empty");
    /// assert_eq!(EMPTY.code(), "non_empty");
    /// ```
    #[must_use]
    pub const fn new(code: &'static str, message: &'static str) -> Self {
        Self { code, message }
    }

    /// Returns the stable, machine-readable error code.
    ///
    /// Use this to branch programmatically; it is intended to stay stable across
    /// releases even when the message changes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use type_lib::ValidationError;
    ///
    /// let err = ValidationError::new("non_empty", "value must not be empty");
    /// assert_eq!(err.code(), "non_empty");
    /// ```
    #[must_use]
    pub const fn code(&self) -> &'static str {
        self.code
    }

    /// Returns the human-readable message.
    ///
    /// Intended for logs and diagnostics, not for matching; the wording may
    /// change between releases.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use type_lib::ValidationError;
    ///
    /// let err = ValidationError::new("non_empty", "value must not be empty");
    /// assert_eq!(err.message(), "value must not be empty");
    /// ```
    #[must_use]
    pub const fn message(&self) -> &'static str {
        self.message
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_preserves_code_and_message() {
        let err = ValidationError::new("non_empty", "value must not be empty");
        assert_eq!(err.code(), "non_empty");
        assert_eq!(err.message(), "value must not be empty");
    }

    #[test]
    fn display_joins_code_and_message() {
        // Format through `core::fmt::Write` into a fixed buffer so the test
        // holds under `no_std` as well as `std`.
        use core::fmt::Write as _;

        let err = ValidationError::new("out_of_range", "expected 1..=10");
        let mut buf = Buf::new();
        let _ = write!(buf, "{err}");
        assert_eq!(buf.as_str(), "out_of_range: expected 1..=10");
    }

    #[test]
    fn equal_errors_compare_equal() {
        let a = ValidationError::new("c", "m");
        let b = ValidationError::new("c", "m");
        let c = ValidationError::new("c", "other");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    /// A tiny fixed-capacity formatter sink, so the `Display` test needs neither
    /// `alloc` nor `std`. It stores up to 64 bytes; the message under test is
    /// shorter than that.
    struct Buf {
        bytes: [u8; 64],
        len: usize,
    }

    impl Buf {
        fn new() -> Self {
            Self {
                bytes: [0; 64],
                len: 0,
            }
        }

        fn as_str(&self) -> &str {
            core::str::from_utf8(&self.bytes[..self.len]).unwrap_or("")
        }
    }

    impl core::fmt::Write for Buf {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            for &byte in s.as_bytes() {
                if self.len < self.bytes.len() {
                    self.bytes[self.len] = byte;
                    self.len += 1;
                }
            }
            Ok(())
        }
    }
}
