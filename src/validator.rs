//! The [`Validator`] trait: reusable, type-level validation rules.

/// A reusable validation rule applied to values of type `T`.
///
/// A `Validator` is a *type-level* predicate. It carries no state and is never
/// instantiated — you implement it on a zero-sized marker type and the rule is
/// selected purely through the type system. Pairing a value with the validator
/// that vouched for it is exactly what [`Refined`](crate::Refined) does, at no
/// runtime cost.
///
/// # Choosing the value type
///
/// `T` is `?Sized`, so a rule can target an unsized type such as `str` or `[u8]`
/// and be reused for the owned forms by being generic over a borrow. The
/// `NonEmpty` rule below works for `str`, `String`, and `&str` alike because it
/// is written over `S: AsRef<str>`:
///
/// ```rust
/// use type_lib::{ValidationError, Validator};
///
/// struct NonEmpty;
///
/// impl<S: AsRef<str> + ?Sized> Validator<S> for NonEmpty {
///     type Error = ValidationError;
///
///     fn validate(value: &S) -> Result<(), Self::Error> {
///         if value.as_ref().is_empty() {
///             Err(ValidationError::new("non_empty", "value must not be empty"))
///         } else {
///             Ok(())
///         }
///     }
/// }
///
/// assert!(NonEmpty::validate("hello").is_ok());
/// assert!(NonEmpty::validate("").is_err());
/// ```
///
/// # Custom error types
///
/// [`Error`](Validator::Error) is an associated type, so a rule can report a
/// rich, structured failure instead of the bundled [`ValidationError`]:
///
/// ```rust
/// use type_lib::Validator;
///
/// #[derive(Debug, PartialEq)]
/// struct TooLong {
///     limit: usize,
///     actual: usize,
/// }
///
/// struct MaxLen8;
///
/// impl Validator<str> for MaxLen8 {
///     type Error = TooLong;
///
///     fn validate(value: &str) -> Result<(), Self::Error> {
///         let actual = value.chars().count();
///         if actual > 8 {
///             Err(TooLong { limit: 8, actual })
///         } else {
///             Ok(())
///         }
///     }
/// }
///
/// assert_eq!(
///     MaxLen8::validate("far-too-long"),
///     Err(TooLong { limit: 8, actual: 12 }),
/// );
/// ```
///
/// [`ValidationError`]: crate::ValidationError
pub trait Validator<T: ?Sized> {
    /// The failure produced when a value violates the rule.
    ///
    /// Use [`ValidationError`](crate::ValidationError) for simple cases, or a
    /// bespoke type when callers need to inspect structured details of the
    /// failure.
    type Error;

    /// Checks `value` against the rule.
    ///
    /// Returns `Ok(())` when the value satisfies the invariant, or
    /// `Err(Self::Error)` describing why it does not. This is a pure predicate:
    /// it never mutates or transforms the value.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`](Validator::Error) when `value` fails the rule.
    fn validate(value: &T) -> Result<(), Self::Error>;
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use crate::ValidationError;

    struct NonEmpty;

    impl<S: AsRef<str> + ?Sized> Validator<S> for NonEmpty {
        type Error = ValidationError;

        fn validate(value: &S) -> Result<(), Self::Error> {
            if value.as_ref().is_empty() {
                Err(ValidationError::new("non_empty", "value must not be empty"))
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn accepts_valid_value() {
        assert!(NonEmpty::validate("hello").is_ok());
    }

    #[test]
    fn rejects_invalid_value() {
        let err = NonEmpty::validate("").unwrap_err();
        assert_eq!(err.code(), "non_empty");
    }

    #[test]
    fn rule_is_reusable_across_borrow_forms() {
        // Same marker type, different value representations.
        assert!(<NonEmpty as Validator<str>>::validate("ok").is_ok());
        let owned = "ok";
        assert!(<NonEmpty as Validator<&str>>::validate(&owned).is_ok());
    }
}
