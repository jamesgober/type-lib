//! Length-based rules for strings, slices, and other measurable values.
//!
//! Each rule here works on any type that implements [`HasLength`]. The crate
//! ships impls for `str` and `[T]` (always), and for `String` and `Vec<T>` when
//! the `alloc` feature is enabled, plus a blanket impl for shared references so a
//! rule applies equally to `str` and `&str`.
//!
//! For strings, "length" is the number of [`char`]s (Unicode scalar values), not
//! the number of bytes — the count a human means by "at most 20 characters". For
//! slices and vectors it is the number of elements.

use crate::{ValidationError, Validator};

/// A value with a measurable length, used by the length rules.
///
/// Implement this for your own container types to make them eligible for
/// [`NonEmpty`], [`MinLen`], [`MaxLen`], and [`LenRange`].
///
/// # Examples
///
/// ```rust
/// use type_lib::rules::HasLength;
///
/// assert_eq!("héllo".length(), 5); // chars, not bytes
/// assert_eq!([1, 2, 3][..].length(), 3);
/// ```
pub trait HasLength {
    /// Returns the length of the value: chars for strings, elements otherwise.
    fn length(&self) -> usize;
}

impl HasLength for str {
    fn length(&self) -> usize {
        self.chars().count()
    }
}

impl<T> HasLength for [T] {
    fn length(&self) -> usize {
        self.len()
    }
}

impl<U: HasLength + ?Sized> HasLength for &U {
    fn length(&self) -> usize {
        U::length(self)
    }
}

#[cfg(feature = "alloc")]
impl HasLength for alloc::string::String {
    fn length(&self) -> usize {
        self.as_str().chars().count()
    }
}

#[cfg(feature = "alloc")]
impl<T> HasLength for alloc::vec::Vec<T> {
    fn length(&self) -> usize {
        self.len()
    }
}

/// Accepts any value whose [`length`](HasLength::length) is greater than zero.
///
/// # Examples
///
/// ```rust
/// use type_lib::rules::NonEmpty;
/// use type_lib::Validator;
///
/// assert!(NonEmpty::validate("hello").is_ok());
/// assert!(NonEmpty::validate("").is_err());
/// assert!(NonEmpty::validate(&[1, 2, 3][..]).is_ok());
/// ```
pub struct NonEmpty;

impl<T: HasLength + ?Sized> Validator<T> for NonEmpty {
    type Error = ValidationError;

    fn validate(value: &T) -> Result<(), Self::Error> {
        if value.length() > 0 {
            Ok(())
        } else {
            Err(ValidationError::new("non_empty", "value must not be empty"))
        }
    }
}

/// Accepts values with at least `MIN` elements (inclusive).
///
/// # Examples
///
/// ```rust
/// use type_lib::rules::MinLen;
/// use type_lib::Validator;
///
/// assert!(MinLen::<3>::validate("abc").is_ok());
/// assert!(MinLen::<3>::validate("ab").is_err());
/// ```
pub struct MinLen<const MIN: usize>;

impl<const MIN: usize, T: HasLength + ?Sized> Validator<T> for MinLen<MIN> {
    type Error = ValidationError;

    fn validate(value: &T) -> Result<(), Self::Error> {
        if value.length() >= MIN {
            Ok(())
        } else {
            Err(ValidationError::new("min_len", "value is too short"))
        }
    }
}

/// Accepts values with at most `MAX` elements (inclusive).
///
/// # Examples
///
/// ```rust
/// use type_lib::rules::MaxLen;
/// use type_lib::Validator;
///
/// assert!(MaxLen::<5>::validate("hello").is_ok());
/// assert!(MaxLen::<5>::validate("too long").is_err());
/// ```
pub struct MaxLen<const MAX: usize>;

impl<const MAX: usize, T: HasLength + ?Sized> Validator<T> for MaxLen<MAX> {
    type Error = ValidationError;

    fn validate(value: &T) -> Result<(), Self::Error> {
        if value.length() <= MAX {
            Ok(())
        } else {
            Err(ValidationError::new("max_len", "value is too long"))
        }
    }
}

/// Accepts values whose length is in the inclusive range `MIN..=MAX`.
///
/// # Examples
///
/// ```rust
/// use type_lib::rules::LenRange;
/// use type_lib::Validator;
///
/// // A typical username constraint: 3 to 16 characters.
/// assert!(LenRange::<3, 16>::validate("alice").is_ok());
/// assert!(LenRange::<3, 16>::validate("ab").is_err());
/// assert!(LenRange::<3, 16>::validate("this-name-is-far-too-long").is_err());
/// ```
pub struct LenRange<const MIN: usize, const MAX: usize>;

impl<const MIN: usize, const MAX: usize, T: HasLength + ?Sized> Validator<T>
    for LenRange<MIN, MAX>
{
    type Error = ValidationError;

    fn validate(value: &T) -> Result<(), Self::Error> {
        let len = value.length();
        if len >= MIN && len <= MAX {
            Ok(())
        } else {
            Err(ValidationError::new(
                "len_range",
                "value length is out of range",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn has_length_counts_chars_and_elements() {
        assert_eq!("héllo".length(), 5);
        assert_eq!([1, 2, 3][..].length(), 3);

        // The blanket impl for references lets a rule see through one `&`.
        let slice: &[u8] = b"abc";
        assert_eq!(slice.length(), 3);
    }

    #[test]
    fn non_empty_rules() {
        assert!(NonEmpty::validate("x").is_ok());
        assert_eq!(NonEmpty::validate("").unwrap_err().code(), "non_empty");
    }

    #[test]
    fn min_max_len_boundaries() {
        assert!(MinLen::<2>::validate("ab").is_ok());
        assert!(MinLen::<2>::validate("a").is_err());
        assert!(MaxLen::<2>::validate("ab").is_ok());
        assert!(MaxLen::<2>::validate("abc").is_err());
    }

    #[test]
    fn len_range_inclusive_bounds() {
        assert!(LenRange::<2, 4>::validate("ab").is_ok());
        assert!(LenRange::<2, 4>::validate("abcd").is_ok());
        assert!(LenRange::<2, 4>::validate("a").is_err());
        assert!(LenRange::<2, 4>::validate("abcde").is_err());
    }
}
