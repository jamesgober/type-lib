//! Numeric rules: sign checks and inclusive integer ranges.
//!
//! Sign rules ([`Positive`], [`NonNegative`], [`Negative`], [`NonPositive`])
//! apply to the signed integer and floating-point primitives. [`InRange`] applies
//! to the integer primitives that fit losslessly in an `i64`
//! (`i8`/`i16`/`i32`/`i64`, `u8`/`u16`/`u32`). [`Finite`] applies to `f32` and
//! `f64`.
//!
//! # Floating-point edge cases
//!
//! Every sign rule rejects NaN, because NaN compares false against zero. Two
//! consequences follow. [`Positive`] and [`NonNegative`] accept positive
//! infinity, and [`Negative`] and [`NonPositive`] accept negative infinity.
//! [`Not`](crate::combinator::Not) inverts the result, so `Not<Positive>`
//! accepts NaN as well as zero and negative values. Combine a sign rule with
//! [`Finite`] (through [`And`](crate::combinator::And)) when NaN and infinity
//! must be rejected:
//!
//! ```rust
//! use type_lib::combinator::And;
//! use type_lib::rules::{Finite, Positive};
//! use type_lib::Refined;
//!
//! type Price = Refined<f64, And<Finite, Positive>>;
//!
//! assert!(Price::new(9.99).is_ok());
//! assert!(Price::new(f64::INFINITY).is_err());
//! assert!(Price::new(f64::NAN).is_err());
//! ```

use crate::{ValidationError, Validator};

/// Accepts strictly positive numbers (`value > 0`).
///
/// # Examples
///
/// ```rust
/// use type_lib::rules::Positive;
/// use type_lib::Validator;
///
/// assert!(Positive::validate(&3).is_ok());
/// assert!(Positive::validate(&0).is_err());
/// assert!(Positive::validate(&-1.5_f64).is_err());
/// ```
pub struct Positive;

/// Accepts non-negative numbers (`value >= 0`).
///
/// # Examples
///
/// ```rust
/// use type_lib::rules::NonNegative;
/// use type_lib::Validator;
///
/// assert!(NonNegative::validate(&0).is_ok());
/// assert!(NonNegative::validate(&-1).is_err());
/// ```
pub struct NonNegative;

/// Accepts strictly negative numbers (`value < 0`).
///
/// # Examples
///
/// ```rust
/// use type_lib::rules::Negative;
/// use type_lib::Validator;
///
/// assert!(Negative::validate(&-2).is_ok());
/// assert!(Negative::validate(&0).is_err());
/// ```
pub struct Negative;

/// Accepts non-positive numbers (`value <= 0`).
///
/// # Examples
///
/// ```rust
/// use type_lib::rules::NonPositive;
/// use type_lib::Validator;
///
/// assert!(NonPositive::validate(&0).is_ok());
/// assert!(NonPositive::validate(&1).is_err());
/// ```
pub struct NonPositive;

macro_rules! impl_sign_rules {
    ($($t:ty),* $(,)?) => {
        $(
            impl Validator<$t> for Positive {
                type Error = ValidationError;

                fn validate(value: &$t) -> Result<(), Self::Error> {
                    if *value > (0 as $t) {
                        Ok(())
                    } else {
                        Err(ValidationError::new("positive", "value must be greater than zero"))
                    }
                }
            }

            impl Validator<$t> for NonNegative {
                type Error = ValidationError;

                fn validate(value: &$t) -> Result<(), Self::Error> {
                    if *value >= (0 as $t) {
                        Ok(())
                    } else {
                        Err(ValidationError::new("non_negative", "value must not be negative"))
                    }
                }
            }

            impl Validator<$t> for Negative {
                type Error = ValidationError;

                fn validate(value: &$t) -> Result<(), Self::Error> {
                    if *value < (0 as $t) {
                        Ok(())
                    } else {
                        Err(ValidationError::new("negative", "value must be less than zero"))
                    }
                }
            }

            impl Validator<$t> for NonPositive {
                type Error = ValidationError;

                fn validate(value: &$t) -> Result<(), Self::Error> {
                    if *value <= (0 as $t) {
                        Ok(())
                    } else {
                        Err(ValidationError::new("non_positive", "value must not be positive"))
                    }
                }
            }
        )*
    };
}

impl_sign_rules!(i8, i16, i32, i64, i128, isize, f32, f64);

/// Accepts finite floating-point numbers: rejects NaN and both infinities.
///
/// Implemented for `f32` and `f64`.
///
/// # Examples
///
/// ```rust
/// use type_lib::rules::Finite;
/// use type_lib::Validator;
///
/// assert!(Finite::validate(&1.5_f64).is_ok());
/// assert!(Finite::validate(&f64::NAN).is_err());
/// assert!(Finite::validate(&f32::NEG_INFINITY).is_err());
/// ```
pub struct Finite;

macro_rules! impl_finite {
    ($($t:ty),* $(,)?) => {
        $(
            impl Validator<$t> for Finite {
                type Error = ValidationError;

                fn validate(value: &$t) -> Result<(), Self::Error> {
                    if value.is_finite() {
                        Ok(())
                    } else {
                        Err(ValidationError::new("finite", "value must be a finite number"))
                    }
                }
            }
        )*
    };
}

impl_finite!(f32, f64);

/// Accepts integers within the inclusive range `MIN..=MAX`.
///
/// Implemented for the integer types that convert losslessly to `i64`
/// (`i8`/`i16`/`i32`/`i64`, `u8`/`u16`/`u32`). The bounds are `i64` const
/// generics, so the rule covers any range expressible in an `i64`.
///
/// `MIN` must not be greater than `MAX`. An inverted range such as
/// `InRange<10, 0>` fails to build when the rule is used, instead of compiling
/// into a type that rejects every value:
///
/// ```compile_fail
/// use type_lib::rules::InRange;
/// use type_lib::Refined;
///
/// let _ = Refined::<i32, InRange<10, 0>>::new(5); // error: MIN > MAX
/// ```
///
/// # Examples
///
/// ```rust
/// use type_lib::rules::InRange;
/// use type_lib::Validator;
///
/// // A percentage: 0 to 100 inclusive.
/// assert!(InRange::<0, 100>::validate(&50_u8).is_ok());
/// assert!(InRange::<0, 100>::validate(&150_i32).is_err());
///
/// // Ranges may be negative.
/// assert!(InRange::<-10, 10>::validate(&-5_i16).is_ok());
/// ```
pub struct InRange<const MIN: i64, const MAX: i64>;

impl<const MIN: i64, const MAX: i64> InRange<MIN, MAX> {
    /// Evaluated when `validate` is instantiated; stops the build if the bounds
    /// are inverted.
    const BOUNDS_ORDERED: () = assert!(MIN <= MAX, "InRange<MIN, MAX> requires MIN <= MAX");
}

macro_rules! impl_in_range {
    ($($t:ty),* $(,)?) => {
        $(
            impl<const MIN: i64, const MAX: i64> Validator<$t> for InRange<MIN, MAX> {
                type Error = ValidationError;

                fn validate(value: &$t) -> Result<(), Self::Error> {
                    // Forces the bound check; older clippy flags `let ()` here.
                    #[allow(clippy::let_unit_value)]
                    let () = Self::BOUNDS_ORDERED;
                    let value = i64::from(*value);
                    if value >= MIN && value <= MAX {
                        Ok(())
                    } else {
                        Err(ValidationError::new("in_range", "value is out of range"))
                    }
                }
            }
        )*
    };
}

impl_in_range!(i8, i16, i32, i64, u8, u16, u32);

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn sign_rules_on_integers() {
        assert!(Positive::validate(&1_i32).is_ok());
        assert!(Positive::validate(&0_i32).is_err());
        assert!(NonNegative::validate(&0_i32).is_ok());
        assert!(NonNegative::validate(&-1_i32).is_err());
        assert!(Negative::validate(&-1_i32).is_ok());
        assert!(Negative::validate(&0_i32).is_err());
        assert!(NonPositive::validate(&0_i32).is_ok());
        assert!(NonPositive::validate(&1_i32).is_err());
    }

    #[test]
    fn sign_rules_on_floats() {
        assert!(Positive::validate(&0.5_f64).is_ok());
        assert!(Positive::validate(&0.0_f64).is_err());
        assert!(Negative::validate(&-0.5_f32).is_ok());
    }

    #[test]
    fn finite_rejects_nan_and_infinities() {
        assert!(Finite::validate(&0.0_f64).is_ok());
        assert!(Finite::validate(&-1.0e300_f64).is_ok());
        assert!(Finite::validate(&f32::MAX).is_ok());
        assert_eq!(Finite::validate(&f64::NAN).unwrap_err().code(), "finite");
        assert!(Finite::validate(&f64::INFINITY).is_err());
        assert!(Finite::validate(&f32::NEG_INFINITY).is_err());
        assert!(Finite::validate(&f32::NAN).is_err());
    }

    #[test]
    fn in_range_single_value_range_is_allowed() {
        assert!(InRange::<7, 7>::validate(&7_i32).is_ok());
        assert!(InRange::<7, 7>::validate(&8_i32).is_err());
    }

    #[test]
    fn in_range_inclusive_bounds_and_codes() {
        assert!(InRange::<0, 100>::validate(&0_u8).is_ok());
        assert!(InRange::<0, 100>::validate(&100_u8).is_ok());
        assert_eq!(
            InRange::<0, 100>::validate(&101_i32).unwrap_err().code(),
            "in_range",
        );
        assert!(InRange::<-5, 5>::validate(&-5_i16).is_ok());
        assert!(InRange::<-5, 5>::validate(&-6_i16).is_err());
    }
}
