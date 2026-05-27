//! End-to-end tests for the parse-dont-validate foundation.
//!
//! These exercise the public API the way a downstream crate would: define a
//! rule, wrap a value, and rely on the type guaranteeing the invariant.

use std::collections::HashSet;

use type_lib::prelude::*;

/// Rejects empty strings. Generic over the borrow so the one rule serves
/// `&str`, `String`, and `str`.
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

/// Accepts integers within an inclusive range, reported with a structured error.
struct Percentage;

#[derive(Debug, PartialEq, Eq)]
struct OutOfRange {
    value: i32,
}

impl Validator<i32> for Percentage {
    type Error = OutOfRange;

    fn validate(value: &i32) -> Result<(), Self::Error> {
        if (0..=100).contains(value) {
            Ok(())
        } else {
            Err(OutOfRange { value: *value })
        }
    }
}

type Username = Refined<String, NonEmpty>;
type Percent = Refined<i32, Percentage>;

#[test]
fn valid_value_constructs() {
    let user = Username::new("alice".to_owned()).expect("non-empty is valid");
    assert_eq!(user.get(), "alice");
    assert_eq!(user.len(), 5); // Deref to String
}

#[test]
fn invalid_value_is_rejected_with_its_error() {
    let err = Username::new(String::new()).unwrap_err();
    assert_eq!(err.code(), "non_empty");

    let err = Percent::new(150).unwrap_err();
    assert_eq!(err, OutOfRange { value: 150 });
}

#[test]
fn refined_round_trips_through_into_inner() {
    let percent = Percent::new(73).expect("in range");
    let raw = percent.into_inner();
    assert_eq!(raw, 73);
    // Re-wrapping a known-good value succeeds again.
    assert!(Percent::new(raw).is_ok());
}

#[test]
fn delegated_traits_enable_collections() {
    // Refined delegates Eq + Hash to the inner value, so it works as a key.
    let mut seen: HashSet<Percent> = HashSet::new();
    let _ = seen.insert(Percent::new(10).unwrap());
    let _ = seen.insert(Percent::new(10).unwrap());
    let _ = seen.insert(Percent::new(20).unwrap());
    assert_eq!(seen.len(), 2);
}

#[test]
fn display_delegates_to_inner() {
    let percent = Percent::new(42).unwrap();
    assert_eq!(format!("{percent}"), "42");
}
