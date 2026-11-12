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

    /// Returns the length if it is at most `limit`, or some value greater than
    /// `limit` otherwise.
    ///
    /// The length rules call this instead of [`length`](HasLength::length), so
    /// a type whose length is expensive to count can stop early. Strings use it
    /// to avoid walking a long input: a string longer than `4 * limit` bytes has
    /// more than `limit` chars, and a shorter one is counted only up to
    /// `limit + 1` chars.
    ///
    /// The default calls [`length`](HasLength::length), which is always
    /// correct. An override must return the exact length whenever that length is
    /// at most `limit`, and a value greater than `limit` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use type_lib::rules::HasLength;
    ///
    /// assert_eq!("héllo".length_at_most(10), 5);
    /// assert!("héllo".length_at_most(3) > 3);
    /// ```
    fn length_at_most(&self, limit: usize) -> usize {
        let _ = limit;
        self.length()
    }
}

/// Char count of `s`, exact up to `limit`, and greater than `limit` otherwise.
fn str_length_at_most(s: &str, limit: usize) -> usize {
    let bytes = s.len();
    if bytes <= limit {
        // At most `bytes` chars, so the exact count is within the limit.
        s.chars().count()
    } else if bytes / 4 > limit {
        // Every char is at most 4 bytes, so there are at least `bytes / 4`
        // chars, which is more than `limit`.
        limit.saturating_add(1)
    } else {
        s.chars().take(limit.saturating_add(1)).count()
    }
}

impl HasLength for str {
    fn length(&self) -> usize {
        self.chars().count()
    }

    fn length_at_most(&self, limit: usize) -> usize {
        str_length_at_most(self, limit)
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

    fn length_at_most(&self, limit: usize) -> usize {
        U::length_at_most(self, limit)
    }
}

#[cfg(feature = "alloc")]
impl HasLength for alloc::string::String {
    fn length(&self) -> usize {
        self.as_str().chars().count()
    }

    fn length_at_most(&self, limit: usize) -> usize {
        str_length_at_most(self.as_str(), limit)
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
        if value.length_at_most(0) > 0 {
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
        if value.length_at_most(MIN) >= MIN {
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
        if value.length_at_most(MAX) <= MAX {
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
///
/// `MIN` must not be greater than `MAX`. An inverted range such as
/// `LenRange<16, 3>` fails to build when the rule is used, instead of compiling
/// into a type that rejects every value:
///
/// ```compile_fail
/// use type_lib::rules::LenRange;
/// use type_lib::Refined;
///
/// let _ = Refined::<&str, LenRange<16, 3>>::new("alice"); // error: MIN > MAX
/// ```
pub struct LenRange<const MIN: usize, const MAX: usize>;

impl<const MIN: usize, const MAX: usize> LenRange<MIN, MAX> {
    /// Evaluated when `validate` is instantiated; stops the build if the bounds
    /// are inverted.
    const BOUNDS_ORDERED: () = assert!(MIN <= MAX, "LenRange<MIN, MAX> requires MIN <= MAX");
}

impl<const MIN: usize, const MAX: usize, T: HasLength + ?Sized> Validator<T>
    for LenRange<MIN, MAX>
{
    type Error = ValidationError;

    fn validate(value: &T) -> Result<(), Self::Error> {
        // Forces the bound check; older clippy flags `let ()` here.
        #[allow(clippy::let_unit_value)]
        let () = Self::BOUNDS_ORDERED;
        // Exact when `len <= MAX`; otherwise some value above `MAX`.
        let len = value.length_at_most(MAX);
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
    fn str_length_at_most_is_exact_up_to_the_limit() {
        // 2-byte, 3-byte, and 4-byte chars around every branch of the helper.
        for s in [
            "",
            "a",
            "abc",
            "h\u{e9}llo",
            "\u{20ac}\u{20ac}\u{20ac}",
            "\u{1f600}\u{1f600}x",
        ] {
            let exact = s.chars().count();
            for limit in 0..12 {
                let got = s.length_at_most(limit);
                if exact <= limit {
                    assert_eq!(got, exact, "{s:?} limit {limit}");
                } else {
                    assert!(got > limit, "{s:?} limit {limit} got {got}");
                }
            }
        }
        assert_eq!("abc".length_at_most(usize::MAX), 3);
    }

    /// A container whose full length must never be asked for: the length rules
    /// must go through `length_at_most`.
    struct Endless;

    impl HasLength for Endless {
        fn length(&self) -> usize {
            panic!("the full length was counted");
        }

        fn length_at_most(&self, limit: usize) -> usize {
            limit.saturating_add(1)
        }
    }

    #[test]
    fn length_rules_stop_at_the_bound() {
        assert!(NonEmpty::validate(&Endless).is_ok());
        assert!(MinLen::<4>::validate(&Endless).is_ok());
        assert!(MaxLen::<4>::validate(&Endless).is_err());
        assert!(LenRange::<1, 4>::validate(&Endless).is_err());
    }

    #[test]
    fn len_range_single_value_range_is_allowed() {
        assert!(LenRange::<2, 2>::validate("ab").is_ok());
        assert!(LenRange::<2, 2>::validate("abc").is_err());
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
