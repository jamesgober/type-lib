//! Combinators for composing [`Validator`]s.
//!
//! [`And`], [`Or`], and [`Not`] build compound rules from simpler ones, entirely
//! at the type level. Because they are themselves [`Validator`]s, they nest:
//! `And<A, Or<B, C>>` is a valid rule.
//!
//! `And` and `Or` require their two sub-rules to share one error type, which is
//! the case for every [built-in rule](crate::rules) (all report
//! [`ValidationError`]). `Not` works with any sub-rule and reports its own
//! [`ValidationError`].
//!
//! # Examples
//!
//! ```rust
//! use type_lib::combinator::And;
//! use type_lib::rules::{Ascii, NonEmpty};
//! use type_lib::Refined;
//!
//! // A token: non-empty and ASCII-only.
//! type Token<'a> = Refined<&'a str, And<NonEmpty, Ascii>>;
//!
//! assert!(Token::new("abc123").is_ok());
//! assert!(Token::new("").is_err()); // fails NonEmpty
//! assert!(Token::new("café").is_err()); // fails Ascii
//! ```

use core::marker::PhantomData;

use crate::{ValidationError, Validator};

/// Passes only when **both** sub-rules `A` and `B` pass.
///
/// `A` is checked first; if it fails its error is returned and `B` is not run.
/// Both sub-rules must share the same [`Validator::Error`] type.
///
/// # Examples
///
/// ```rust
/// use type_lib::combinator::And;
/// use type_lib::rules::{MaxLen, NonEmpty};
/// use type_lib::Validator;
///
/// type ShortNonEmpty = And<NonEmpty, MaxLen<8>>;
///
/// assert!(ShortNonEmpty::validate("ok").is_ok());
/// assert!(ShortNonEmpty::validate("").is_err()); // NonEmpty fails first
/// assert!(ShortNonEmpty::validate("way too long").is_err()); // MaxLen fails
/// ```
pub struct And<A, B>(PhantomData<fn() -> (A, B)>);

impl<T, A, B, E> Validator<T> for And<A, B>
where
    T: ?Sized,
    A: Validator<T, Error = E>,
    B: Validator<T, Error = E>,
{
    type Error = E;

    fn validate(value: &T) -> Result<(), Self::Error> {
        A::validate(value)?;
        B::validate(value)
    }
}

/// Passes when **either** sub-rule `A` or `B` passes.
///
/// `A` is checked first; if it passes, `B` is not run. If `A` fails, the result
/// of `B` is returned — so when both fail, the error is `B`'s. Both sub-rules
/// must share the same [`Validator::Error`] type.
///
/// # Examples
///
/// ```rust
/// use type_lib::combinator::Or;
/// use type_lib::rules::{Alphanumeric, Trimmed};
/// use type_lib::Validator;
///
/// type AlnumOrTrimmed = Or<Alphanumeric, Trimmed>;
///
/// assert!(AlnumOrTrimmed::validate("abc123").is_ok()); // alphanumeric
/// assert!(AlnumOrTrimmed::validate("a b c").is_ok());  // trimmed
/// assert!(AlnumOrTrimmed::validate(" oops ").is_err()); // neither
/// ```
pub struct Or<A, B>(PhantomData<fn() -> (A, B)>);

impl<T, A, B, E> Validator<T> for Or<A, B>
where
    T: ?Sized,
    A: Validator<T, Error = E>,
    B: Validator<T, Error = E>,
{
    type Error = E;

    fn validate(value: &T) -> Result<(), Self::Error> {
        match A::validate(value) {
            Ok(()) => Ok(()),
            Err(_) => B::validate(value),
        }
    }
}

/// Passes exactly when the sub-rule `A` **fails**.
///
/// `Not` inverts any [`Validator`], regardless of its error type, and reports a
/// [`ValidationError`] with code `"not"` when `A` unexpectedly passes.
///
/// # Examples
///
/// ```rust
/// use type_lib::combinator::Not;
/// use type_lib::rules::Ascii;
/// use type_lib::Validator;
///
/// // Require that a string is *not* pure ASCII.
/// type NonAscii = Not<Ascii>;
///
/// assert!(NonAscii::validate("café").is_ok()); // not ASCII -> passes
/// assert!(NonAscii::validate("plain").is_err()); // ASCII -> fails
/// ```
pub struct Not<A>(PhantomData<fn() -> A>);

impl<T, A> Validator<T> for Not<A>
where
    T: ?Sized,
    A: Validator<T>,
{
    type Error = ValidationError;

    fn validate(value: &T) -> Result<(), Self::Error> {
        match A::validate(value) {
            Ok(()) => Err(ValidationError::new(
                "not",
                "value matched a rule it was required to violate",
            )),
            Err(_) => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use crate::rules::{Ascii, MaxLen, NonEmpty};

    #[test]
    fn and_requires_both() {
        type R = And<NonEmpty, MaxLen<4>>;
        assert!(R::validate("abc").is_ok());
        assert_eq!(R::validate("").unwrap_err().code(), "non_empty");
        assert_eq!(R::validate("toolong").unwrap_err().code(), "max_len");
    }

    #[test]
    fn or_accepts_either() {
        type R = Or<NonEmpty, MaxLen<0>>;
        assert!(R::validate("x").is_ok()); // NonEmpty passes
        assert!(R::validate("").is_ok()); // NonEmpty fails, but MaxLen<0> passes on ""
    }

    #[test]
    fn or_returns_second_error_when_both_fail() {
        use crate::rules::MinLen;

        type R = Or<MinLen<5>, MinLen<10>>;
        // "abc" (len 3) fails both bounds; Or returns the second rule's error.
        assert_eq!(R::validate("abc").unwrap_err().code(), "min_len");
    }

    #[test]
    fn not_inverts() {
        type R = Not<Ascii>;
        assert!(R::validate("café").is_ok());
        assert_eq!(R::validate("ascii").unwrap_err().code(), "not");
    }
}
